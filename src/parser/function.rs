use super::{
    utils::{node_to_string, type_name_to_string},
    Function, FunctionArg, SchemaId,
};
use crate::{MigrationPlanner, MigrationResult, NodeDiff, NodeItem};
use itertools::Itertools;
use pg_query::{protobuf::CreateFunctionStmt, Node, NodeEnum, NodeRef};

impl NodeItem for Function {
    type Inner = CreateFunctionStmt;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn type_name(&self) -> &'static str {
        "function"
    }

    fn node(&self) -> &NodeEnum {
        &self.node
    }

    fn inner(&self) -> anyhow::Result<&Self::Inner> {
        match &self.node {
            NodeEnum::CreateFunctionStmt(stmt) => Ok(stmt),
            _ => anyhow::bail!("not a create function statement"),
        }
    }

    fn revert(&self) -> anyhow::Result<NodeEnum> {
        let sql = format!("DROP FUNCTION {}", self.signature());
        let parsed = pg_query::parse(&sql)?;
        let node = parsed.protobuf.nodes()[0].0;
        match node {
            NodeRef::DropStmt(stmt) => Ok(NodeEnum::DropStmt(stmt.clone())),
            _ => anyhow::bail!("not a drop statement"),
        }
    }
}

impl TryFrom<&CreateFunctionStmt> for Function {
    type Error = anyhow::Error;
    fn try_from(stmt: &CreateFunctionStmt) -> Result<Self, Self::Error> {
        let args = parse_args(&stmt.parameters);

        let id = stmt
            .funcname
            .iter()
            .filter_map(node_to_string)
            .join(".")
            .parse()?;

        let returns = type_name_to_string(stmt.return_type.as_ref().unwrap());

        let node = NodeEnum::CreateFunctionStmt(stmt.clone());
        Ok(Self {
            id,
            args,
            returns,
            node,
        })
    }
}

impl MigrationPlanner for NodeDiff<Function> {
    type Migration = String;

    fn drop(&self) -> MigrationResult<Self::Migration> {
        if let Some(old) = &self.old {
            let sqls = vec![old.revert()?.deparse()?];
            Ok(sqls)
        } else {
            Ok(vec![])
        }
    }

    fn create(&self) -> MigrationResult<Self::Migration> {
        if let Some(new) = &self.new {
            let sqls = vec![new.node.deparse()?];
            Ok(sqls)
        } else {
            Ok(vec![])
        }
    }

    fn alter(&self) -> MigrationResult<Self::Migration> {
        match (&self.old, &self.new) {
            (Some(old), Some(new)) => {
                // if args or return type changed, drop and create
                if old.args != new.args || old.returns != new.returns {
                    return Ok(vec![]);
                }

                let sql = new.node.deparse()?;
                let sql = sql.replace("CREATE FUNCTION", "CREATE OR REPLACE FUNCTION");
                Ok(vec![sql])
            }
            _ => Ok(vec![]),
        }
    }
}

impl Function {
    pub fn signature(&self) -> String {
        format!(
            "{}({})",
            self.id,
            self.args.iter().map(|a| &a.data_type).join(", ")
        )
    }
}

#[allow(dead_code)]
fn parse_id(nodes: &[Node], args: &[FunctionArg]) -> SchemaId {
    let mut names = nodes.iter().filter_map(node_to_string).collect::<Vec<_>>();
    assert!(!names.is_empty() && names.len() <= 2);
    let name = names.pop().unwrap();
    let func_name = format!("{}({})", name, args.iter().map(|a| &a.data_type).join(", "));
    names.push(func_name);
    SchemaId::new_with(&names.iter().map(|v| v.as_str()).collect::<Vec<_>>())
}

fn parse_args(args: &[Node]) -> Vec<FunctionArg> {
    args.iter()
        .map(|n| match n.node.as_ref() {
            Some(NodeEnum::FunctionParameter(param)) => FunctionArg {
                name: param.name.clone(),
                data_type: type_name_to_string(param.arg_type.as_ref().unwrap()),
            },
            _ => panic!("not a function parameter"),
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use crate::{Differ, MigrationPlanner};

    use super::*;

    #[test]
    fn valid_create_function_sql_should_parse() {
        let f1 = "CREATE FUNCTION test(name text, value integer) RETURNS text LANGUAGE sql STABLE AS $$ select 1 $$";
        let fun: Function = f1.parse().unwrap();
        assert_eq!(fun.id, SchemaId::new("public", "test"));
        assert_eq!(
            fun.args,
            vec![
                FunctionArg {
                    name: "name".to_string(),
                    data_type: "text".to_string()
                },
                FunctionArg {
                    name: "value".to_string(),
                    data_type: "pg_catalog.int4".to_string()
                },
            ]
        );
        assert_eq!(fun.returns, "text");
    }

    #[test]
    fn unchanged_function_should_return_none() {
        let f1 = "CREATE FUNCTION public.test() RETURNS text LANGUAGE sql STABLE AS $$ select 1 $$";
        let f2 = "CREATE FUNCTION public.test() RETURNS text LANGUAGE sql STABLE AS $$ select 1 $$";
        let old: Function = f1.parse().unwrap();
        let new: Function = f2.parse().unwrap();
        let diff = old.diff(&new).unwrap();
        assert!(diff.is_none());
    }

    #[test]
    fn function_add_new_args_should_be_treated_as_new_function() {
        let f1 = "CREATE FUNCTION test() RETURNS text LANGUAGE SQL stable AS $$ select 1 $$";
        let f2 = "CREATE FUNCTION test(name1 text) RETURNS text LANGUAGE sql STABLE AS $$ select name1 $$";
        let old: Function = f1.parse().unwrap();
        let new: Function = f2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0], "DROP FUNCTION public.test()");
        assert_eq!(plan[1], f2);
    }

    #[test]
    fn function_change_arg_type_should_generate_migration() {
        let f1 = "CREATE FUNCTION test(name1 text) RETURNS text LANGUAGE sql STABLE AS $$ select name1 $$";
        let f2 = "CREATE FUNCTION test(name1 int4) RETURNS int4 LANGUAGE sql STABLE AS $$ select name1 $$";
        let old: Function = f1.parse().unwrap();
        let new: Function = f2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0], "DROP FUNCTION public.test(text)");
        assert_eq!(plan[1], f2);
    }

    #[test]
    fn function_change_content_should_generate_migration() {
        let f1 = "CREATE FUNCTION test(name1 text) RETURNS text LANGUAGE sql STABLE AS $$ select name1 $$";
        let f2 = "CREATE FUNCTION test(name2 text) RETURNS text LANGUAGE sql IMMUTABLE AS $$ select name2 $$";
        let old: Function = f1.parse().unwrap();
        let new: Function = f2.parse().unwrap();
        let diff = old.diff(&new).unwrap().unwrap();
        let plan = diff.plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0], "CREATE OR REPLACE FUNCTION test(name2 text) RETURNS text LANGUAGE sql IMMUTABLE AS $$ select name2 $$");
    }
}
