#[macro_export]
macro_rules! translate {
    ( $i18:expr, $id:expr, $( $name:ident : $value:expr ),* ) => {
        {
            let mut params_map = HashMap::new();
            $(
                params_map.insert(stringify!($name), $value.to_string());
            )*
            $i18.translate_with_params($id, params_map)
        }
    };

    ( $i18:expr, $id:expr ) => {
        {
            $i18.translate($id, HashMap::new())
        }
    };
}
