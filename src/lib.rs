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

impl<K, V> PartialEq<K> for KV<K, V> where K: Eq {
  fn eq(&self, other_k: &K) -> bool {
    self.k.eq(&other_k)
  }
}

impl<K, V> PartialEq for KV<K, V> where K: Eq {
  fn eq(&self, other: &KV<K, V>) -> bool {
    self.k.eq(&other.k)
  }
}

impl<K, V> PartialOrd<K> for KV<K, V> where K: Ord {
  fn partial_cmp(&self, other_k: &K) -> Option<Ordering> {
    Some(self.k.cmp(&other_k))
  }
}

impl<K, V> PartialOrd for KV<K, V> where K: Ord {
  fn partial_cmp(&self, other: &KV<K, V>) -> Option<Ordering> {
    Some(self.k.cmp(&other.k))
  }
}

pub struct VertreapMap<K, V, P=usize> {
  vtreap:   Vertreap<KV<K, V>, P>,
}

impl<K, V, P> Default for VertreapMap<K, V, P> {
  fn default() -> VertreapMap<K, V, P> {
    VertreapMap{
      vtreap:   Vertreap::default(),
    }
  }
}

impl<K, V, P> Clone for VertreapMap<K, V, P> {
  fn clone(&self) -> VertreapMap<K, V, P> {
    VertreapMap{
      vtreap:   self.vtreap.clone(),
    }
  }
}

impl<K, V, P> VertreapMap<K, V, P> {
  pub fn new() -> VertreapMap<K, V, P> {
    VertreapMap{
      vtreap:   Vertreap::new(),
    }
  }
}

impl<K, P> VertreapMap<K, P> {
  pub fn len(&self) -> usize {
    self.vtreap.len()
  }
}

impl<K, V, P> VertreapMap<K, V, P>
where K: Ord,
{
  pub fn find(&self, key: &K) -> Option<Rc<KV<K, V>>> {
    self.vtreap.find(key)
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
    let new_vtreap = self.vtreap.append_with_priority(priority, KV{k: key, v: val});
    VertreapMap{
      vtreap:   new_vtreap,
    }
  }
}

pub struct VertreapSet<K, P=usize> {
  vtreap:   Vertreap<K, P>,
}

impl<K, P> Default for VertreapSet<K, P> {
  fn default() -> VertreapSet<K, P> {
    VertreapSet{
      vtreap:   Vertreap::default(),
    }
  }
}

impl<K, P> Clone for VertreapSet<K, P> {
  fn clone(&self) -> VertreapSet<K, P> {
    VertreapSet{
      vtreap:   self.vtreap.clone(),
    }
  }
}

impl<K, P> VertreapSet<K, P> {
  pub fn new() -> VertreapSet<K, P> {
    VertreapSet{
      vtreap:   Vertreap::new(),
    }
  }
}

impl<K, P> VertreapSet<K, P> {
  pub fn len(&self) -> usize {
    self.vtreap.len()
  }
}

impl<K, P> VertreapSet<K, P>
where K: Ord,
{
  pub fn contains(&self, key: &K) -> bool {
    self.vtreap.find(key).is_some()
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
    let new_vtreap = self.vtreap.append_with_priority(priority, key);
    VertreapSet{
      vtreap:   new_vtreap,
    }
  }
}

pub struct VertreapIter<Item, P> {
  done:     bool,
  next:     Option<Rc<VertreapNode<Item, P>>>,
  stack:    Vec<Rc<VertreapNode<Item, P>>>,
}

impl<Item, P> VertreapIter<Item, P> {
  pub fn new(root: Option<Rc<VertreapNode<Item, P>>>) -> VertreapIter<Item, P> {
    VertreapIter{
      done:     false,
      next:     root,
      stack:    Vec::new(),
    }
  }
}

impl<Item, P> Iterator for VertreapIter<Item, P> {
  type Item = Rc<VertreapNode<Item, P>>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.done {
      return None;
    }
    let mut retval = None;
    let mut do_break = false;
    while !do_break {
      self.next = match self.next.take() {
        Some(next_node) => {
          let left = next_node.left.clone();
          self.stack.push(next_node);
          left
        }
        None => {
          match self.stack.pop() {
            Some(top_node) => {
              let right = top_node.right.clone();
              retval = Some(top_node);
              do_break = true;
              right
            }
            None => {
              self.done = true;
              do_break = true;
              None
            }
          }
        }
      };
    }
    retval
  }
}

struct VertreapState {
  version:  Cell<usize>,
}

pub struct Vertreap<Item, P=usize> {
  version:  usize,
  count:    usize,
  state:    Rc<VertreapState>,
  root:     Option<Rc<VertreapNode<Item, P>>>,
}

impl<Item, P> Default for Vertreap<Item, P> {
  fn default() -> Vertreap<Item, P> {
    Vertreap::new()
  }
}

impl<Item, P> Clone for Vertreap<Item, P> {
  fn clone(&self) -> Vertreap<Item, P> {
    Vertreap{
      version:  self.version,
      count:    self.count,
      state:    self.state.clone(),
      root:     self.root.clone(),
    }
  }
}

impl<Item, P> Vertreap<Item, P> {
  pub fn new() -> Vertreap<Item, P> {
    Vertreap{
      version:  0,
      count:    0,
      state:    Rc::new(VertreapState{version: Cell::new(0)}),
      root:     None,
    }
  }

  pub fn len(&self) -> usize {
    self.count
  }

  pub fn find<K>(&self, key: &K) -> Option<Rc<Item>> where Item: PartialOrd<K> {
    match self.root {
      None => None,
      Some(ref root_node) => root_node._find(self.version, key),
    }
  }
}

impl<Item, P> Vertreap<Item, P>
where Item: PartialOrd,
      P: Copy + Ord,
{
  pub fn append_with_priority(&self, priority: P, item: Item) -> Vertreap<Item, P> {
    let old_version = self.state.version.get();
    let new_version = old_version + 1;
    assert!(new_version != 0);
    self.state.version.set(new_version);
    assert!(self.version < new_version);
    let (new_root, new_ct) = match self.root {
      None => {
        (VertreapNode::leaf(new_version, priority, item), 1)
      }
      Some(ref root_node) => {
        root_node._append(new_version, priority, item)
      }
    };
    let new_vtreap = Vertreap{
      version:    new_version,
      count:      self.count + new_ct,
      state:      self.state.clone(),
      root:       Some(Rc::new(new_root)),
    };
    new_vtreap
  }
}

pub struct VertreapNode<Item, P> {
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

impl<Item, P> VertreapNode<Item, P> where Item: PartialOrd, P: Copy + Ord {
  fn _append(&self, new_version: usize, new_priority: P, new_item: Item) -> (VertreapNode<Item, P>, usize) {
    assert!(self.version < new_version);
    match new_item.partial_cmp(&*self.item) {
      None => panic!(),
      Some(Ordering::Equal) => {
        (VertreapNode::branch(new_version, self.priority, Rc::new(new_item), self.left.clone(), self.right.clone()), 0)
      }
      Some(Ordering::Less) => {
        let (new_left, new_ct) = match self.left {
          None => (VertreapNode::leaf(new_version, new_priority, new_item), 1),
          Some(ref l_node) => l_node._append(new_version, new_priority, new_item),
        };
        let heap_ordered = new_left.priority <= self.priority;
        let tmp_node = VertreapNode::branch(new_version, self.priority, self.item.clone(), Some(Rc::new(new_left)), self.right.clone());
        if heap_ordered {
          (tmp_node, new_ct)
        } else {
          (tmp_node._rotate_right(new_version), new_ct)
        }
      }
      Some(Ordering::Greater) => {
        let (new_right, new_ct) = match self.right {
          None => (VertreapNode::leaf(new_version, new_priority, new_item), 1),
          Some(ref r_node) => r_node._append(new_version, new_priority, new_item),
        };
        let heap_ordered = new_right.priority <= self.priority;
        let tmp_node = VertreapNode::branch(new_version, self.priority, self.item.clone(), self.left.clone(), Some(Rc::new(new_right)));
        if heap_ordered {
          (tmp_node, new_ct)
        } else {
          (tmp_node._rotate_left(new_version), new_ct)
        }
      }
    }
  }
}
