use super::{
    utils::{get_type_name, node_to_string},
    Function, FunctionArg, SchemaId,
};
use crate::NodeItem;
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
        let sql = format!("DROP FUNCTION {}", self.id);
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

        let id = parse_id(&stmt.funcname, &args);

        let returns = get_type_name(&stmt.return_type.as_ref().unwrap().names);

        let node = NodeEnum::CreateFunctionStmt(stmt.clone());
        Ok(Self {
            id,
            args,
            returns,
            node,
        })
    }
}

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
                data_type: get_type_name(&param.arg_type.as_ref().unwrap().names),
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
        assert_eq!(
            fun.id,
            SchemaId::new("public", "test(text, pg_catalog.int4)")
        );
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
        let diff = old.diff(&new).unwrap_err();
        assert_eq!(
            diff.to_string(),
            "can't diff public.test() and public.test(text)"
        );
    }

    #[test]
    fn function_change_arg_name_should_generate_migration() {
        let f1 = "CREATE FUNCTION test(name1 text) RETURNS text LANGUAGE sql STABLE AS $$ select name1 $$";
        let f2 = "CREATE FUNCTION test(name2 text) RETURNS text LANGUAGE sql STABLE AS $$ select name2 $$";
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
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0], "DROP FUNCTION public.test(text)");
        assert_eq!(plan[1], f2);
    }
}
