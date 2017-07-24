//! Routing for paths delimited by a forward slash, with the ability to capture
//! specified path segments.
//!
//! # Example
//!
//! ```rust
//! use path_router::Tree;
//! let mut routes = Tree::new();
//! routes.add("GET/user/:username/profile", "profile.html");
//! assert_eq!(
//!     routes.find("GET/user/my_name/profile"),
//!     Some((&"profile.html", vec![("username", String::from("my_name"))])));
//! ```

#[macro_use] extern crate log;

/// The routing information is stored as a trie.
///
/// # Description
///
/// Each node is labelled with its path segment. The value is a tuple. The first
/// element is generic, and is usually a handler function; the second element is
/// the captured path segments.
///
pub struct Tree<'a, T> {
    label: &'a str,
    value: Option<(T, Vec<&'a str>)>,
    branches: Vec<Tree<'a, T>>
}

impl<'a, T> Tree<'a, T> {
    /// Constructs a new routing tree.
    pub fn new<'b>() -> Tree<'b, T> {
        Tree {
            label: "",
            value: None,
            branches: Vec::new()
        }
    }
    /// Adds a new path and its associated value to the tree. Prefix a segment
    /// with a colon (:) to enable capturing on the segment.
    ///
    /// # Panics
    ///
    /// Panics if a duplicate route is added.
    ///
    pub fn add(&mut self, key: &'a str, value: T) {
        info!("Adding route {}", key);
        let segments = key.split('/').filter(|s| !s.is_empty());
        let capture_labels = Vec::new();    // Will be filled while iterating
        self.add_(segments, value, capture_labels);
    }
    fn add_<I: Iterator<Item=&'a str>>(
        &mut self, mut segments: I, value: T,
        mut capture_labels: Vec<&'a str>) {
        match segments.next() {
            None => {
                if self.value.is_some() {
                    error!("Duplicate route!");
                    panic!("Duplicate route!");
                }
                self.value = Some((value, capture_labels))
            },
            Some(segment) => {
                if let Some(existing_branch) =
                    self.branches.iter_mut().find(|t| t.label == segment) {
                        existing_branch.add_(segments, value, capture_labels);
                        return;
                    }
                if segment.starts_with(':') {
                    capture_labels.push(&segment[1..]);
                    if let Some(existing_branch) =
                        self.branches.iter_mut().find(|t| t.label.is_empty()) {
                            existing_branch.add_(
                                segments, value, capture_labels);
                            return;
                        }
                    let mut branch = Tree {
                        label: "",
                        value: None,
                        branches: Vec::new()
                    };
                    branch.add_(segments, value, capture_labels);
                    self.branches.push(branch);
                } else {
                    let mut branch = Tree {
                        label: segment,
                        value: None,
                        branches: Vec::new()
                    };
                    branch.add_(segments, value, capture_labels);
                    self.branches.push(branch);
                }
            }
        }
    }
    /// Retrieve the value associated with the path, together with the captured
    /// path segments.
    pub fn find(&self, key: &str) -> Option<(&T, Vec<(&'a str, String)>)> {
        let segments: Vec<&str> = key.split('/')
            .filter(|s| !s.is_empty())
            .collect();
        let mut captured = Vec::new();  // Will be filled while iterating
        self.find_(segments.as_slice(), &mut captured)
            .map(|&(ref v, ref labels)| {
                (v, labels.iter().cloned().zip(captured).collect())
            })
    }
    fn find_(&self, segments: &[&str],
             captured: &mut Vec<String>) -> Option<&(T, Vec<&'a str>)> {
        match segments.split_first() {
            None => self.value.as_ref(),
            Some((&segment, remaining)) => self.branches.iter().filter_map(|t| {
                if t.label == segment {
                    t.find_(remaining, captured)
                } else if t.label == "" {
                    captured.push(String::from(segment));
                    let result = t.find_(remaining, captured);
                    if result.is_none() {
                        captured.pop();
                    }
                    result
                } else {
                    None
                }
            }).next()
        }
    }
}

#[cfg(test)]
mod tests {
    use Tree;
    #[test]
    fn can_add_and_find() {
        let mut routes = Tree::new();
        routes.add("/", 0);
        routes.add("/var", 1);
        routes.add("/var/www", 11);
        routes.add("/var/log", 12);
        assert_eq!(routes.find("/vax"), None);
        assert_eq!(routes.find("/var/something"), None);
        assert_eq!(
            routes.find("////"),
            Some((&0, vec![])));
        assert_eq!(
            routes.find("//var//"),
            Some((&1, vec![])));
        assert_eq!(
            routes.find("/var/www/"),
            Some((&11, vec![])));
        assert_eq!(
            routes.find("/var/log/"),
            Some((&12, vec![])));
    }
    #[test]
    fn can_add_and_capture_and_find() {
        let mut routes = Tree::new();
        routes.add("/user/:username", 11);
        routes.add("/user/:username/:intent/show", 111);
        routes.add("/user/:username/profile", 112);
        assert_eq!(routes.find("/user/myname/delete"), None);
        assert_eq!(routes.find("/user/myname/cook/throw"), None);
        assert_eq!(
            routes.find("/user/myname"),
            Some((&11, vec![("username", String::from("myname"))])));
        assert_eq!(
            routes.find("/user/myname/profile"),
            Some((&112, vec![("username", String::from("myname"))])));
        assert_eq!(
            routes.find("/user/myname/cook/show"),
            Some((&111, vec![
                  ("username", String::from("myname")),
                  ("intent", String::from("cook"))
            ])));
    }
    #[test]
    fn can_add_and_capture_and_find_handlers() {
        let mut routes = Tree::new();
        let handler = |captured: Vec<(&str, String)>| {
            assert_eq!(captured.len(), 2);
            assert_eq!(captured[0].0, "folder");
            assert_eq!(captured[0].1, "myfolder");
            assert_eq!(captured[1].0, "file");
            assert_eq!(captured[1].1, "myfile");
        };
        routes.add("home/:folder/:file", handler);
        match routes.find("/home/myfolder/myfile") {
            None => assert!(false),
            Some((fx, captured)) => fx(captured)
        }
    }
}
