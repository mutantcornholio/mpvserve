use std::path::Path;

pub fn get_urlencoded_path(path_from_root: &Path) -> Option<String> {
    let mut res = String::from("");

    for path_item in path_from_root.iter() {
        let item = path_item.to_str()?;
        res += "/";
        res += &String::from(urlencoding::encode(item));
    }

    Some(res)
}
