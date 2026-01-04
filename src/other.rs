
#[allow(non_snake_case)]
fn ASCII_filter(in_string: &[u8]) -> Option<String> {
    let filteredstring: Vec<u8> = in_string
        .iter()
        .filter_map(|&c| {
            if (c >= 32 && c <= 127) || c == 13 || c == 10 {
                Some(c)
            } else {
                None
            }
        })
        .collect();
    String::from_utf8(filteredstring).ok()
}

macro_rules! tuple_as {
    ($t: expr, $ty: ident) => {{
        let (a, b) = $t;
        let a = a as $ty;
        let b = b as $ty;
        (a, b)
    }};
    ($t: expr, ($ty: ident)) => {{
        let (a, b) = $t;
        let a = a as $ty;
        let b = b as $ty;
        (a, b)
    }};
    ($t: expr, ($($ty: ident),*)) => {{
        let ($($ty,)*) = $t;
        ($($ty as $ty,)*)
    }}}
    