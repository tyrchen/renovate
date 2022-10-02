#[macro_export]
macro_rules! map_insert_schema {
    ($map:expr, $item:ident) => {
        $map.entry($item.id.schema.clone())
            .or_insert(Default::default())
            .insert($item.id.name.clone(), $item);
    };
}

#[macro_export]
macro_rules! map_insert_relation {
    ($map:expr, $item:ident) => {
        $map.entry($item.id.schema_id.clone())
            .or_insert(Default::default())
            .insert($item.id.name.clone(), $item);
    };
}

#[macro_export]
macro_rules! map_insert {
    ($map:expr, $item:ident) => {
        $map.insert($item.id.clone(), $item);
    };
}
