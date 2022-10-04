use super::{utils::parsec::parse_single_priv, Privilege, SinglePriv};
use crate::{parser::SchemaId, DiffItem, MigrationPlanner, NodeDiff};
use anyhow::Context;
use debug_ignore::DebugIgnore;
use pg_query::{
    protobuf::{AccessPriv, GrantStmt, GrantTargetType, ObjectType},
    Node, NodeEnum, NodeRef,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

impl FromStr for SinglePriv {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let (_, p) =
            parse_single_priv(s).map_err(|_| anyhow::anyhow!("invalid single priv: {}", s))?;
        Ok(p)
    }
}

impl From<SinglePriv> for AccessPriv {
    fn from(p: SinglePriv) -> Self {
        let cols = p
            .cols
            .into_iter()
            .map(|s| NodeEnum::String(pg_query::protobuf::String { str: s }))
            .map(|n| Node { node: Some(n) })
            .collect::<Vec<_>>();
        AccessPriv {
            priv_name: p.name,
            cols,
        }
    }
}

impl From<AccessPriv> for SinglePriv {
    fn from(p: AccessPriv) -> Self {
        let name = p.priv_name;
        let cols: BTreeSet<String> = p
            .cols
            .into_iter()
            .filter_map(|n| {
                n.node.and_then(|c| match c {
                    NodeEnum::String(s) => Some(s.str),
                    _ => None,
                })
            })
            .collect();
        Self { name, cols }
    }
}

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
            let mut v = old.clone();
            v.grant = !v.grant;
            Ok(Some(format!("{};", v.node.deparse()?)))
        } else {
            Ok(None)
        }
    }

    fn create(&self) -> anyhow::Result<Option<Self::Migration>> {
        if let Some(new) = &self.new {
            Ok(Some(format!("{};", new.node.deparse()?)))
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
                {
                    // we can't alter these privilege changes, so we need to drop and recreate it
                    return Ok(None);
                }
                // let added = new.privileges.difference(&old.privileges);
                todo!()
            }
            _ => Ok(None),
        }
    }
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
}
