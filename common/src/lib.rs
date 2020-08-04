use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::slice::Iter;
use std::str::FromStr;
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Corpus {
    pub intent_mapping: IntentMapping,
    pub cases: Vec<Case>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntentMapping {
    pub val: HashMap<String, String>, // key=intent, value=domain
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Case {
    pub text: String,
    pub reference: usize,
    pub count: usize,
    pub gold: Vec<Annotation>,
    pub left: Vec<Annotation>,
    pub right: Vec<Annotation>,
    #[serde(skip)]
    pub gold_vs_left: AnnotationComparison,
    #[serde(skip)]
    pub gold_vs_right: AnnotationComparison,
    #[serde(skip)]
    pub left_vs_right: AnnotationComparison,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Annotation {
    #[serde(skip)]
    pub domain: String,
    pub intent: String,
    pub values: Vec<(String, String)>,
}

impl Annotation {
    pub fn empty() -> Self {
        Annotation {
            domain: "".to_string(),
            intent: "".to_string(),
            values: vec![],
        }
    }
}

enum_str! {
    AnnotationComparison,
    (Different,"different"),
    (SameDomains,"same domains"),
    (SameIntents,"same intents"),
    (SameProperties,"same properties"),
    (SameValues,"same values"),

}

impl Default for AnnotationComparison {
    fn default() -> Self {
        AnnotationComparison::Different
    }
}

fn annotation_dist(a: &Annotation, b: &Annotation) -> u32 {
    if a == b {
        return 0;
    };
    if a.intent != b.intent {
        return 1000;
    };
    let aligned_values = kv_align(&a.values, &b.values);

    aligned_values.iter().fold(0, |acc, (s, _, _)| acc + *s)
}

fn kv_dist(a: &(String, String), b: &(String, String)) -> u32 {
    if a == b {
        return 0;
    }
    if a.0 != b.0 {
        return 100;
    } // different properties
    10 // same property, different values
}

pub fn annotation_align(
    a: &[Annotation],
    b: &[Annotation],
) -> Vec<(u32, Option<Annotation>, Option<Annotation>)> {
    let (smallest, mut largest) = if a.len() <= b.len() {
        (a, b.to_owned())
    } else {
        (b, a.to_owned())
    };
    let a_is_smallest = a.len() <= b.len();
    let mut result: Vec<(u32, Option<Annotation>, Option<Annotation>)> = vec![];

    for x in smallest {
        let (best_index, distance) =
            largest
                .iter()
                .enumerate()
                .fold((0, 9999), |(min_index, min), (index, val)| {
                    let d = annotation_dist(x, &val);
                    if d < min {
                        (index, d)
                    } else {
                        (min_index, min)
                    }
                }); // find best match for x in largest, and also returns distance
        if distance > 1000 {
            if a_is_smallest {
                result.push((100, Some(x.clone()), None));
            } else {
                result.push((100, None, Some(x.clone())));
            }
        } else {
            let best_match = largest.swap_remove(best_index); // remove best match from future candidates
            if a_is_smallest {
                result.push((distance, Some(x.clone()), Some(best_match)));
            } else {
                result.push((distance, Some(best_match), Some(x.clone())));
            }
        }
    }
    // Add remainders from largest
    for x in &largest {
        if a_is_smallest {
            result.push((100, None, Some(x.clone())));
        } else {
            result.push((100, Some(x.clone()), None));
        }
    }
    result
}

pub fn kv_align(
    a: &[(String, String)],
    b: &[(String, String)],
) -> Vec<(u32, Option<(String, String)>, Option<(String, String)>)> {
    let (smallest, mut largest) = if a.len() <= b.len() {
        (a, b.to_owned())
    } else {
        (b, a.to_owned())
    };
    let a_is_smallest = a.len() <= b.len();
    let mut result: Vec<(u32, Option<(String, String)>, Option<(String, String)>)> = vec![];

    for x in smallest {
        let (best_index, distance) =
            largest
                .iter()
                .enumerate()
                .fold((0, 9999), |(min_index, min), (index, val)| {
                    let d = kv_dist(x, &val);
                    if d < min {
                        (index, d)
                    } else {
                        (min_index, min)
                    }
                }); // find best match for x in largest, and also returns distance
        if distance >= 100 {
            if a_is_smallest {
                result.push((distance, Some(x.clone()), None));
            } else {
                result.push((distance, None, Some(x.clone())));
            }
        } else {
            let best_match = largest.swap_remove(best_index); // remove best match from future candidates
            if a_is_smallest {
                result.push((distance, Some(x.clone()), Some(best_match)));
            } else {
                result.push((distance, Some(best_match), Some(x.clone())));
            }
        }
    }
    // Add remainders from largest
    for x in &largest {
        if a_is_smallest {
            result.push((100, None, Some(x.clone())));
        } else {
            result.push((100, Some(x.clone()), None));
        }
    }
    result
}

pub enum CompareMode {
    All,
    Any,
}

pub fn compare(a: &[Annotation], b: &[Annotation]) -> AnnotationComparison {
    let aligned_annotations = annotation_align(a, b);
    let mut result = vec![];

    for (d, a, b) in aligned_annotations.into_iter() {
        match (a, b) {
            (None, None) => panic!("not possible"),
            (Some(_a), None) => result.push(AnnotationComparison::Different),
            (None, Some(_b)) => result.push(AnnotationComparison::Different),
            (Some(a), Some(b)) => {
                if a.intent != b.intent {
                    if a.domain != b.domain {
                        result.push(AnnotationComparison::Different);
                    } else {
                        result.push(AnnotationComparison::SameDomains);
                    }
                } else {
                    match d {
                        x if x == 0 => result.push(AnnotationComparison::SameValues),
                        x if x < 100 => result.push(AnnotationComparison::SameProperties),
                        _ => result.push(AnnotationComparison::SameIntents),
                    }
                }
            }
        }
    }
    // find worst performance
    result
        .iter()
        .fold(AnnotationComparison::SameValues, |worst, &x| {
            if x < worst {
                x
            } else {
                worst
            }
        })
}

impl Corpus {
    pub fn empty() -> Self {
        Corpus {
            intent_mapping: IntentMapping {
                val: HashMap::new(),
            },
            cases: vec![],
        }
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
                        $name::$key => $value
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
