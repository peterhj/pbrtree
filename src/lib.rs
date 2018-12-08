extern crate rand;

use rand::prelude::{Rng, thread_rng};
use rand::distributions::{Distribution, Standard};

use std::cell::{Cell};
use std::cmp::{Ordering};
use std::rc::{Rc};

pub struct KV<K, V> {
  pub k: K,
  pub v: V,
}

impl<K, V> PartialEq<K> for KV<K, V> where K: PartialEq {
  fn eq(&self, other_k: &K) -> bool {
    self.k.eq(&other_k)
  }
}

impl<K, V> PartialEq for KV<K, V> where K: PartialEq {
  fn eq(&self, other: &KV<K, V>) -> bool {
    self.k.eq(&other.k)
  }
}

impl<K, V> Eq for KV<K, V> where K: Eq {
}

impl<K, V> PartialOrd<K> for KV<K, V> where K: PartialOrd {
  fn partial_cmp(&self, other_k: &K) -> Option<Ordering> {
    self.k.partial_cmp(&other_k)
  }
}

impl<K, V> PartialOrd for KV<K, V> where K: PartialOrd {
  fn partial_cmp(&self, other: &KV<K, V>) -> Option<Ordering> {
    self.k.partial_cmp(&other.k)
  }
}

impl<K, V> Ord for KV<K, V> where K: Ord {
  fn cmp(&self, other: &KV<K, V>) -> Ordering {
    self.k.cmp(&other.k)
  }
}

pub struct VertreapMap<K, V, P=usize> {
  version:  usize,
  state:    Rc<VertreapState>,
  root:     Option<Rc<VertreapNode<KV<K, V>, P>>>,
}

impl<K, V, P> Default for VertreapMap<K, V, P> {
  fn default() -> VertreapMap<K, V, P> {
    VertreapMap::new()
  }
}

impl<K, V, P> Clone for VertreapMap<K, V, P> {
  fn clone(&self) -> VertreapMap<K, V, P> {
    VertreapMap{
      version:  self.version,
      state:    self.state.clone(),
      root:     self.root.clone(),
    }
  }
}

impl<K, V, P> VertreapMap<K, V, P> {
  pub fn new() -> VertreapMap<K, V, P> {
    VertreapMap{
      version:  0,
      state:    Rc::new(VertreapState{version: Cell::new(0)}),
      root:     None,
    }
  }
}

impl<K, V, P> VertreapMap<K, V, P>
where K: Ord,
{
  pub fn find(&self, key: &K) -> Option<Rc<KV<K, V>>> {
    match self.root {
      None => None,
      Some(ref root_node) => root_node._find(self.version, key),
    }
  }
}

impl<K, V, P> VertreapMap<K, V, P>
where K: Ord,
      P: Copy + Ord,
      Standard: Distribution<P>,
{
  pub fn append(&self, key: K, val: V) -> VertreapMap<K, V, P> {
    self.append_with_rng(key, val, &mut thread_rng())
  }

  pub fn append_with_rng<R: Rng>(&self, key: K, val: V, rng: &mut R) -> VertreapMap<K, V, P> {
    let priority: P = rng.sample(&Standard);
    self.append_with_priority(priority, key, val)
  }

  fn append_with_priority(&self, priority: P, key: K, val: V) -> VertreapMap<K, V, P> {
    let old_version = self.state.version.get();
    let new_version = old_version + 1;
    assert!(new_version != 0);
    self.state.version.set(new_version);
    assert!(self.version < new_version);
    let new_root = match self.root {
      None => {
        VertreapNode::leaf(new_version, priority, KV{k: key, v: val})
      }
      Some(ref root_node) => {
        root_node._append(new_version, priority, KV{k: key, v: val})
      }
    };
    let new_map = VertreapMap{
      version:    new_version,
      state:      self.state.clone(),
      root:       Some(Rc::new(new_root)),
    };
    new_map
  }
}

pub struct VertreapSet<K, P=usize> {
  version:  usize,
  state:    Rc<VertreapState>,
  root:     Option<Rc<VertreapNode<K, P>>>,
}

impl<K, P> Default for VertreapSet<K, P> {
  fn default() -> VertreapSet<K, P> {
    VertreapSet::new()
  }
}

impl<K, P> Clone for VertreapSet<K, P> {
  fn clone(&self) -> VertreapSet<K, P> {
    VertreapSet{
      version:  self.version,
      state:    self.state.clone(),
      root:     self.root.clone(),
    }
  }
}

impl<K, P> VertreapSet<K, P> {
  pub fn new() -> VertreapSet<K, P> {
    VertreapSet{
      version:  0,
      state:    Rc::new(VertreapState{version: Cell::new(0)}),
      root:     None,
    }
  }
}

impl<K, P> VertreapSet<K, P>
where K: Ord,
{
  pub fn contains(&self, key: &K) -> bool {
    match self.root {
      None => false,
      Some(ref root_node) => root_node._find(self.version, key).is_some(),
    }
  }
}

impl<K, P> VertreapSet<K, P>
where K: Ord,
      P: Copy + Ord,
      Standard: Distribution<P>,
{
  pub fn append(&self, key: K) -> VertreapSet<K, P> {
    self.append_with_rng(key, &mut thread_rng())
  }

  pub fn append_with_rng<R: Rng>(&self, key: K, rng: &mut R) -> VertreapSet<K, P> {
    let priority: P = rng.sample(&Standard);
    self.append_with_priority(priority, key)
  }

  fn append_with_priority(&self, priority: P, key: K) -> VertreapSet<K, P> {
    let old_version = self.state.version.get();
    let new_version = old_version + 1;
    assert!(new_version != 0);
    self.state.version.set(new_version);
    assert!(self.version < new_version);
    let new_root = match self.root {
      None => {
        VertreapNode::leaf(new_version, priority, key)
      }
      Some(ref root_node) => {
        root_node._append(new_version, priority, key)
      }
    };
    let new_set = VertreapSet{
      version:    new_version,
      state:      self.state.clone(),
      root:       Some(Rc::new(new_root)),
    };
    new_set
  }
}

struct VertreapState {
  version:  Cell<usize>,
}

struct VertreapNode<Item, P> {
  version:  usize,
  priority: P,
  item:     Rc<Item>,
  left:     Option<Rc<VertreapNode<Item, P>>>,
  right:    Option<Rc<VertreapNode<Item, P>>>,
}

impl<Item, P> VertreapNode<Item, P> {
  fn leaf(version: usize, priority: P, item: Item) -> VertreapNode<Item, P> {
    VertreapNode{
      version,
      priority,
      item:     Rc::new(item),
      left:     None,
      right:    None,
    }
  }

  fn branch(version: usize, priority: P, item: Rc<Item>, left: Option<Rc<VertreapNode<Item, P>>>, right: Option<Rc<VertreapNode<Item, P>>>) -> VertreapNode<Item, P> {
    if let Some(ref left_node) = left {
      assert!(left_node.version <= version);
    }
    if let Some(ref right_node) = right {
      assert!(right_node.version <= version);
    }
    VertreapNode{
      version,
      priority,
      item,
      left,
      right,
    }
  }
}

impl<Item, P> VertreapNode<Item, P> {
  fn _find<K>(&self, version: usize, key: &K) -> Option<Rc<Item>> where Item: PartialOrd<K> {
    assert!(self.version <= version);
    match self.item.partial_cmp(key) {
      None => panic!(),
      Some(Ordering::Equal) => {
        Some(self.item.clone())
      }
      Some(Ordering::Greater) => {
        match self.left {
          None => None,
          Some(ref l_node) => l_node._find(version, key),
        }
      }
      Some(Ordering::Less) => {
        match self.right {
          None => None,
          Some(ref r_node) => r_node._find(version, key),
        }
      }
    }
  }
}

impl<Item, P> VertreapNode<Item, P> where P: Copy {
  fn _rotate_left(&self, new_version: usize) -> VertreapNode<Item, P> {
    assert!(self.version <= new_version);
    if let Some(ref l_node) = self.left {
      assert!(l_node.version <= new_version);
    }
    let old_right = match self.right {
      None => panic!(),
      Some(ref r_node) => {
        assert!(r_node.version <= new_version);
        if let Some(ref rl_node) = r_node.left {
          assert!(rl_node.version <= new_version);
        }
        if let Some(ref rr_node) = r_node.right {
          assert!(rr_node.version <= new_version);
        }
        r_node.clone()
      }
    };
    let new_left = VertreapNode::branch(new_version, self.priority, self.item.clone(), self.left.clone(), old_right.left.clone());
    let new_up = VertreapNode::branch(new_version, old_right.priority, old_right.item.clone(), Some(Rc::new(new_left)), old_right.right.clone());
    new_up
  }

  fn _rotate_right(&self, new_version: usize) -> VertreapNode<Item, P> {
    assert!(self.version <= new_version);
    if let Some(ref r_node) = self.right {
      assert!(r_node.version <= new_version);
    }
    let old_left = match self.left {
      None => panic!(),
      Some(ref l_node) => {
        assert!(l_node.version <= new_version);
        if let Some(ref ll_node) = l_node.left {
          assert!(ll_node.version <= new_version);
        }
        if let Some(ref lr_node) = l_node.right {
          assert!(lr_node.version <= new_version);
        }
        l_node.clone()
      }
    };
    let new_right = VertreapNode::branch(new_version, self.priority, self.item.clone(), old_left.right.clone(), self.right.clone());
    let new_up = VertreapNode::branch(new_version, old_left.priority, old_left.item.clone(), old_left.left.clone(), Some(Rc::new(new_right)));
    new_up
  }
}

impl<Item, P> VertreapNode<Item, P> where Item: Ord, P: Copy + Ord {
  fn _append(&self, new_version: usize, new_priority: P, new_item: Item) -> VertreapNode<Item, P> {
    assert!(self.version < new_version);
    match new_item.cmp(&*self.item) {
      Ordering::Equal => {
        let new_node = VertreapNode::branch(new_version, self.priority, Rc::new(new_item), self.left.clone(), self.right.clone());
        new_node
      }
      Ordering::Less => {
        let new_left = match self.left {
          None => VertreapNode::leaf(new_version, new_priority, new_item),
          Some(ref l_node) => l_node._append(new_version, new_priority, new_item),
        };
        let heap_ordered = new_left.priority <= self.priority;
        let tmp_node = VertreapNode::branch(new_version, self.priority, self.item.clone(), Some(Rc::new(new_left)), self.right.clone());
        if heap_ordered {
          tmp_node
        } else {
          tmp_node._rotate_right(new_version)
        }
      }
      Ordering::Greater => {
        let new_right = match self.right {
          None => VertreapNode::leaf(new_version, new_priority, new_item),
          Some(ref r_node) => r_node._append(new_version, new_priority, new_item),
        };
        let heap_ordered = new_right.priority <= self.priority;
        let tmp_node = VertreapNode::branch(new_version, self.priority, self.item.clone(), self.left.clone(), Some(Rc::new(new_right)));
        if heap_ordered {
          tmp_node
        } else {
          tmp_node._rotate_left(new_version)
        }
      }
    }
  }
}
