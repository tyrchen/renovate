fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let content = std::fs::read_to_string(filename).unwrap();
    let result = pg_query::parse(&content).unwrap();
    let node = result.protobuf.nodes()[0].0.to_enum();
    println!("{:#?}", node);
    let sql = node.deparse().unwrap();
    println!(
        "{}",
        sqlformat::format(&sql, &Default::default(), Default::default())
    );
}
