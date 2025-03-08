pub mod dom;

use dom::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        let _ = elem(
            "span".to_string(),
            HashMap::new(),
            vec![text("Hello World".to_string())],
        );
    }
}
