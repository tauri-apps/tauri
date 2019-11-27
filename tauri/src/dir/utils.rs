pub fn get_dir_name_from_path(path: String) -> String {
  let path_collect: Vec<&str> = path.split("/").collect();
  return path_collect[path_collect.len() - 1].to_string();
}
