mod single_priv;

use super::{Privilege, SinglePriv};
use crate::{parser::SchemaId, DiffItem, MigrationPlanner, NodeDelta, NodeDiff};
use anyhow::Context;
use debug_ignore::DebugIgnore;
use pg_query::{
    protobuf::{GrantStmt, GrantTargetType, ObjectType},
    Node, NodeEnum, NodeRef,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

impl DiffItem for Privilege {
    fn id(&self) -> String {
        format!("{}:{}", self.id, self.grantee)
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }
}

impl FromStr for Privilege {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let parsed =
            pg_query::parse(s).with_context(|| format!("Failed to parse grant/revoke: {}", s))?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::GrantStmt(stmt) => Self::try_from(stmt),
            _ => anyhow::bail!("not an index: {}", s),
        }
    }
}

impl TryFrom<&GrantStmt> for Privilege {
    type Error = anyhow::Error;
    fn try_from(stmt: &GrantStmt) -> Result<Self, Self::Error> {
        let target_type = get_target_type(stmt);
        let object_type = get_object_type(stmt)?;
        let id = get_id(stmt)?;
        let privileges = get_privileges(stmt);
        let grantee = get_grantee(stmt);
        let node = NodeEnum::GrantStmt(stmt.clone());
        Ok(Self {
            target_type,
            object_type,
            id,
            privileges,
            grantee,
            node: DebugIgnore(node),
            grant: stmt.is_grant,
        })
    }
}

impl MigrationPlanner for NodeDiff<Privilege> {
    type Migration = String;

    fn drop(&self) -> anyhow::Result<Option<Self::Migration>> {
        if let Some(old) = &self.old {
            let sql = gen_grant_sql(&old.node, None, !old.grant)?;

            Ok(Some(format!("{};", sql)))
        } else {
            Ok(None)
        }
    }

    fn create(&self) -> anyhow::Result<Option<Self::Migration>> {
        if let Some(new) = &self.new {
            let sql = gen_grant_sql(&new.node, None, new.grant)?;
            Ok(Some(format!("{};", sql)))
        } else {
            Ok(None)
        }
    }

    fn alter(&self) -> anyhow::Result<Option<Vec<Self::Migration>>> {
        match (&self.old, &self.new) {
            (Some(old), Some(new)) => {
                if old.grant != new.grant
                    || old.target_type != new.target_type
                    || old.grantee != new.grantee
                    || old.privileges.is_empty()
                    || new.privileges.is_empty()
                {
                    // we can't alter these privilege changes, so we need to drop and recreate it
                    return Ok(None);
                }
                let delta = difference(&old.privileges, &new.privileges);
                let mut migrations = Vec::new();
                for removed in delta.removed {
                    let sql = gen_grant_sql(&old.node, Some(removed), false)?;
                    migrations.push(format!("{};", sql));
                }

                for added in delta.added {
                    let sql = gen_grant_sql(&old.node, Some(added), true)?;
                    migrations.push(format!("{};", sql));
                }

                for (v1, v2) in delta.changed {
                    let sql = gen_grant_sql(&old.node, Some(v1), false)?;
                    migrations.push(format!("{};", sql));
                    let sql = gen_grant_sql(&new.node, Some(v2), true)?;
                    migrations.push(format!("{};", sql));
                }
                Ok(Some(migrations))
            }
            _ => Ok(None),
        }
    }
}

fn difference(
    old: &BTreeMap<String, SinglePriv>,
    new: &BTreeMap<String, SinglePriv>,
) -> NodeDelta<SinglePriv> {
    let mut delta = NodeDelta::default();

    let old_keys: BTreeSet<_> = old.keys().collect();
    let new_keys: BTreeSet<_> = new.keys().collect();
    let added = new_keys.difference(&old_keys);
    let removed = old_keys.difference(&new_keys);
    let might_changed = old_keys.intersection(&new_keys);

    for key in added {
        delta.added.insert(new[*key].clone());
    }

    for key in removed {
        delta.removed.insert(old[*key].clone());
    }

    for key in might_changed {
        let old_priv = &old[*key];
        let new_priv = &new[*key];
        if old_priv != new_priv {
            delta.changed.insert((old_priv.clone(), new_priv.clone()));
        }
    }

    delta
}

fn gen_grant_sql(node: &NodeEnum, sp: Option<SinglePriv>, grant: bool) -> anyhow::Result<String> {
    let mut stmt = match node {
        NodeEnum::GrantStmt(stmt) => stmt.clone(),
        _ => anyhow::bail!("not a grant statement"),
    };
    stmt.is_grant = grant;
    if let Some(sp) = sp {
        stmt.privileges = vec![Node {
            node: Some(NodeEnum::AccessPriv(sp.into())),
        }];
    }
    Ok(NodeEnum::GrantStmt(stmt).deparse()?)
}

fn get_target_type(stmt: &GrantStmt) -> GrantTargetType {
    let target_type = GrantTargetType::from_i32(stmt.targtype);
    assert!(target_type.is_some());
    target_type.unwrap()
}

fn get_object_type(stmt: &GrantStmt) -> anyhow::Result<ObjectType> {
    let object_type = ObjectType::from_i32(stmt.objtype);
    assert!(object_type.is_some());
    match object_type.unwrap() {
        ObjectType::ObjectTable => Ok(ObjectType::ObjectTable),
        ObjectType::ObjectSchema => Ok(ObjectType::ObjectSchema),
        v => anyhow::bail!("unsupported grant/revoke object type: {:?}", v),
    }
}

fn get_id(stmt: &GrantStmt) -> anyhow::Result<String> {
    // pg_dump generated grant would only have one object
    assert!(stmt.objects.len() == 1);
    let node = &stmt.objects[0].node;
    assert!(node.is_some());
    let name = match node.as_ref().unwrap() {
        NodeEnum::String(s) => s.str.clone(),
        NodeEnum::RangeVar(v) => SchemaId::from(v).to_string(),
        _ => anyhow::bail!("unsupported grant/revoke object name: {:?}", node),
    };

    Ok(name)
}

fn get_privileges(stmt: &GrantStmt) -> BTreeMap<String, SinglePriv> {
    stmt.privileges
        .iter()
        .filter_map(|p| {
            p.node.as_ref().and_then(|v| match v {
                NodeEnum::AccessPriv(p) => {
                    let p = SinglePriv::from(p.clone());
                    Some((p.name.clone(), p))
                }
                _ => None,
            })
        })
        .collect()
}

fn get_grantee(stmt: &GrantStmt) -> String {
    let name = stmt.grantees.first().and_then(|n| match n.node.as_ref() {
        Some(NodeEnum::RoleSpec(r)) => Some(r.rolename.clone()),
        _ => None,
    });
    assert!(name.is_some());
    name.unwrap()
}

#[cfg(test)]
mod tests {
    use crate::Differ;

    use super::*;

    #[test]
    fn grant_all_should_parse() {
        let s = "GRANT ALL ON TABLE public.test TO test";
        let p = Privilege::from_str(s).unwrap();
        assert!(p.grant);
        assert_eq!(p.target_type, GrantTargetType::AclTargetObject);
        assert_eq!(p.object_type, ObjectType::ObjectTable);
        assert_eq!(p.id, "public.test");
        assert_eq!(p.grantee, "test");
        assert_eq!(p.privileges.len(), 0);
    }

    #[test]
    fn grant_partial_should_parse() {
        let s = "GRANT SELECT(id, name), UPDATE(name) ON TABLE public.test TO test";
        let p = Privilege::from_str(s).unwrap();
        assert!(p.grant);
        assert_eq!(p.target_type, GrantTargetType::AclTargetObject);
        assert_eq!(p.object_type, ObjectType::ObjectTable);
        assert_eq!(p.id, "public.test");
        assert_eq!(p.grantee, "test");
        assert_eq!(p.privileges.len(), 2);
        assert_eq!(
            p.privileges["select"].cols,
            vec!["id".into(), "name".into()].into_iter().collect()
        );
        assert_eq!(
            p.privileges["update"].cols,
            vec!["name".into(),].into_iter().collect()
        );
    }

    #[test]
    fn grand_diff_change_to_all_should_work() {
        let s1 = "GRANT SELECT(id, name) ON TABLE public.test TO test";
        let s2 = "GRANT ALL ON TABLE public.test TO test";
        let p1 = Privilege::from_str(s1).unwrap();
        let p2 = Privilege::from_str(s2).unwrap();
        let diff = p1.diff(&p2).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(
            plan[0],
            "REVOKE select (id, name) ON public.test FROM test;"
        );
        assert_eq!(plan[1], "GRANT ALL ON public.test TO test;");
    }

    #[test]
    fn grand_diff_change_owner_should_work() {
        let s1 = "GRANT SELECT(id, name) ON TABLE public.test TO test";
        let s2 = "GRANT SELECT(id, name) ON TABLE public.test TO test1";
        let p1 = Privilege::from_str(s1).unwrap();
        let p2 = Privilege::from_str(s2).unwrap();
        let diff = p1.diff(&p2).unwrap_err();

        assert_eq!(
            diff.to_string(),
            "can't diff public.test:test and public.test:test1"
        );
    }

    #[test]
    fn grant_diff_create_should_work() {
        let s1 = "GRANT SELECT(id, name) ON TABLE public.test TO test";
        let s2 = "GRANT SELECT(id, name), UPDATE(name) ON TABLE public.test TO test";
        let p1 = Privilege::from_str(s1).unwrap();
        let p2 = Privilege::from_str(s2).unwrap();
        let diff = p1.diff(&p2).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0], "GRANT update (name) ON public.test TO test;");
    }

    #[test]
    fn grant_diff_drop_should_work() {
        let s1 = "GRANT SELECT(id, name), DELETE(name) ON TABLE public.test TO test";
        let s2 = "GRANT SELECT(id, name) ON TABLE public.test TO test";
        let p1 = Privilege::from_str(s1).unwrap();
        let p2 = Privilege::from_str(s2).unwrap();
        let diff = p1.diff(&p2).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0], "REVOKE delete (name) ON public.test FROM test;");
    }

    #[test]
    fn grant_diff_alter_should_work() {
        let s1 = "GRANT SELECT(id, name), DELETE(name) ON TABLE public.test TO test";
        let s2 = "GRANT SELECT(id, temp), UPDATE(name) ON TABLE public.test TO test";
        let p1 = Privilege::from_str(s1).unwrap();
        let p2 = Privilege::from_str(s2).unwrap();
        let diff = p1.diff(&p2).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 4);
        assert_eq!(plan[0], "REVOKE delete (name) ON public.test FROM test;");
        assert_eq!(plan[1], "GRANT update (name) ON public.test TO test;");
        assert_eq!(
            plan[2],
            "REVOKE select (id, name) ON public.test FROM test;"
        );
        assert_eq!(plan[3], "GRANT select (id, temp) ON public.test TO test;");
    }
}
