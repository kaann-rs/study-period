use crate::Args;
use std::path::PathBuf;

#[test]
fn test_get_dir() {
    let args: Args = Args {
        width: 10,
        height: 10,
        output: PathBuf::new(),
        name: String::from("bora"),
        path: String::from("test-empty2"),
    };

    assert!(args.get_dir().is_err());
}
