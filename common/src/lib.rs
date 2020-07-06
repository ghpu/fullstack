use bimap::BiMap;
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use std::ops::Range;
use std::str::FromStr;
use std::vec::Vec;


#[derive(Serialize, Deserialize, Debug)]
pub struct Annotation {
    pub intent: String,
    pub values: Vec<(String,String)>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Case {
    pub text: String,
    pub gold: Vec<Annotation>,
    pub a : Vec<Annotation>,
    pub b: Vec<Annotation>
}

#[derive(Deserialize, Debug)]
pub struct IntentMapping {
    pub val: HashMap<String,String> // key=intent, value=domain
}


#[derive(Deserialize, Debug)]
pub struct DataFromFile {
    pub name: String,
}

/*
 * A graph representation in Rust
 * All graph elements are owned by a HashSet in a Graph object
 * View are computed with references to these graph elements
 * */

#[derive(Deserialize, Debug)]
pub struct Graph {
    names: BiMap<Path, Id>,
    elements: HashMap<Id, Element>,
    dependents: HashMap<Id, HashSet<Id>>, // Parent , List of depending nodes
}

impl Graph {
    fn new() -> Graph {
        return Graph {
            names: BiMap::new(),
            elements: HashMap::new(),
            dependents: HashMap::new(),
        };
    }

    fn contains_path(&self, path: &Path) -> bool {
        self.names.contains_left(path)
    }

    fn contains_id(&self, id: &Id) -> bool {
        self.names.contains_right(id)
    }

    fn add(&mut self, path: &Path, value: ElementValue) -> Result<Element, GraphError> {
        if self.contains_path(path) {
            return Err(GraphError::AlreadyExists);
        }
        let id = NonZeroUsize::new(self.names.len() + 1).unwrap(); // 1-based new id

        let e = Element {
            id: id,
            value: value.clone(),
        };
        match self.elements.insert(id.clone(), e.clone()) {
            None => (),
            Some(e) => return Err(GraphError::Error),
        }

        match value {
            ElementValue::LinkValue(l) => {
                let mut dependents = self
                    .dependents
                    .remove(&id.clone())
                    .unwrap_or(HashSet::new());
                dependents.insert(l.from);
                dependents.insert(l.to);
                self.dependents.insert(id.clone(), dependents);
            }
            _ => (),
        }

        Ok(e)
    }
}

pub type Id = NonZeroUsize;

#[derive(PartialOrd, PartialEq, Eq, Clone, Hash, Debug, Deserialize)]
pub struct Path(Vec<String>);

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub enum ElementValue {
    NodeValue(Node),
    LinkValue(Link),
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub enum Content {
    Local(String),
    Remote(Id, Range<usize>),
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct Node {
    content: Content,
    labels: HashSet<String>,
    values: HashMap<String, String>,
    comments: Vec<String>,
}

impl Node {
    fn empty() -> Node {
        Node {
            content: Content::Local("".to_string()),
            labels: HashSet::new(),
            values: HashMap::new(),
            comments: vec![],
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct Link {
    from: Id,
    to: Id,
    labels: HashSet<String>,
    values: HashMap<String, String>,
    comments: Vec<String>,
}

/** There can be only one Element for a given id, so we instanciate special version of PartialEq
 * and Hash for Element */
#[derive(Eq, Debug, Clone, Deserialize)]
pub struct Element {
    id: Id,
    value: ElementValue,
}

impl From<Node> for ElementValue {
    fn from(node: Node) -> Self {
        ElementValue::NodeValue(node)
    }
}

impl From<Link> for ElementValue {
    fn from(link: Link) -> Self {
        ElementValue::LinkValue(link)
    }
}

impl PartialEq for Element {
    fn eq(&self, other: &Element) -> bool {
        self.id.eq(&other.id)
    }
}

impl std::hash::Hash for Element {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, PartialEq, Eq)]
enum GraphError {
    AlreadyExists,
    DontExist,
    HasParents,
    Error,
}
