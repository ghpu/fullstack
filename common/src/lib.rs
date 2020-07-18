use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap};
use std::str::FromStr;
use std::vec::Vec;
use std::slice::Iter;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Corpus {
    pub intent_mapping: IntentMapping,
    pub cases: Vec<Case>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntentMapping {
    pub val: HashMap<String,String> // key=intent, value=domain
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Case {
    pub text: String,
    pub reference: usize,
    pub count: usize,
    pub gold: Vec<Annotation>,
    pub left : Vec<Annotation>,
    pub right: Vec<Annotation>,
    #[serde(skip)]
    pub gold_vs_left: Vec<AnnotationComparison>,
    #[serde(skip)]
    pub gold_vs_right: Vec<AnnotationComparison>,
    #[serde(skip)]
    pub left_vs_right: Vec<AnnotationComparison>,
}


#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord )]
pub struct Annotation {
    #[serde(skip)]
    pub domain: String,
    pub intent: String,
    pub values: Vec<(String,String)>
}


enum_str!{
    AnnotationComparison,
    (Different,"different"),
    (SameDomains,"same domains"),
    (SameIntents,"same intents"),
    (SameProperties,"same properties"),
    (SameValues,"same values"),

}

impl Default for AnnotationComparison {
    fn default() -> Self { AnnotationComparison::Different }
}

pub fn compare(a: &Vec<Annotation>, b: &Vec<Annotation>) -> Vec<AnnotationComparison> {
    let mut result = AnnotationComparison::Different;
    let zipped = a.iter().zip(b.iter());
    if a.len() == b.len() && zipped.clone().all(|(c,d)| c.domain == d.domain) {
        result = AnnotationComparison::SameDomains;
    }
    if a.len() == b.len() && zipped.clone().all(|(c,d)| c.intent==d.intent) {
        result = AnnotationComparison::SameIntents;
        if zipped.clone().all(|(c,d)| if c.values.len() != d.values.len() {false}else{c.values.iter().zip(d.values.iter()).all(|(e,f)| e == f )}) {
            result = AnnotationComparison::SameValues;
        }
        else if zipped.clone().all(|(c,d)| if c.values.len() != d.values.len() {false}else{c.values.iter().zip(d.values.iter()).all(|(e,f)| e.0 == f.0 )}) {
            result = AnnotationComparison::SameProperties;
        } 
    }

    vec!(result)
}


impl Corpus {
    pub fn empty() -> Self {
        Corpus{intent_mapping: IntentMapping {val:HashMap::new()}, cases:vec![]}
    }
}

pub trait AsStr {
    fn as_str(&self) -> &str;
}

#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

#[macro_export]
macro_rules! enum_str {
    ($name:ident, $(($key:ident, $value:expr),)*) => {
        #[derive(Clone,Copy,Hash,Debug,PartialOrd,Ord,PartialEq,Eq)]
        pub enum $name
        {
            $($key),*
        }


        impl AsStr for $name {
            fn as_str(&self) -> &str {
                match self {
                    $(
                        &$name::$key => $value
                     ),*
                }
            }
        }

        impl FromStr for $name {
            type Err = ();

            fn from_str(val: &str) -> Result<Self,Self::Err> {
                match val
                {
                    $(
                        $value => Ok($name::$key)
                     ),*,
                    _ => Err(())
                }
            }
        }
        impl $name {
            pub fn iterator() -> Iter<'static, $name> {
                static VALUES: [$name; count!($($key)*)] = [$($name::$key),*];
                VALUES.iter()
            }
        }

    }
}

