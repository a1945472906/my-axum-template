// use std::collections::linked_list::{LinkedList};
use std::collections::HashMap;
use std::hash::Hash;
use std::ptr::NonNull;
use std::boxed::Box;
use std::marker::PhantomData;
use std::ops::Drop;
use std::mem;

pub struct Node<T> {
    elem: T, 
    prev: Option<NonNull<Node<T>>>,
    next: Option<NonNull<Node<T>>>
}
impl <T> Node<T> {
    pub fn new(elem: T) -> Self {
        Self { elem, prev: None, next: None }
    }
}

pub struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    marker: PhantomData<Box<Node<T>>>
}
impl <T> LinkedList<T> {
    pub fn new() -> Self{
        LinkedList { head: None, tail: None, len: 0, marker: PhantomData }
    }
    pub fn push_front_node(&mut self, mut node: NonNull<Node<T>>) {
        unsafe {
            node.as_mut().next = self.head;
        };
        match self.head {
            Some(mut head) => {
                unsafe { head.as_mut().prev = Some(node); }
            },
            None => {
                self.tail = Some(node)
            }
        }
        self.head = Some(node);
        self.len += 1;
    }
    
    pub fn pop_back_node(&mut self) -> Option<Node<T>> {
        self.tail.map(|node| {
            let node = unsafe { Box::from_raw(node.as_ptr()) };
            self.tail = node.prev;
            match self.tail {
                None => self.head = None,
                Some(mut tail) => {
                    unsafe {
                        tail.as_mut().next = None;
                    }
                }
            }
            self.len -= 1;
            *node
        })
    }
    
    pub fn pop_front_node(&mut self) -> Option<Node<T>> {
        self.head.map(|node| {
            let node = unsafe {Box::from_raw(node.as_ptr())};
            self.head = node.next;
            match self.head {
                None => self.tail = None,
                Some(mut head) => {
                    unsafe {
                        head.as_mut().prev = None;
                    }
                }
            }
            self.len -= 1;
            *node
        })
    }

    pub fn unlink_node(&mut self, mut node: NonNull<Node<T>>) {
        let node = unsafe {node.as_mut()};
        match node.prev {
            Some(prev) => unsafe {
                (*prev.as_ptr()).next = node.next;
            },
            None => {
                self.head = node.next;
            }
        };
        match node.next {
            Some(next) => unsafe {
                (*next.as_ptr()).prev = node.prev;
            },
            None => {
                self.tail = node.prev;
            }
        }
        self.len -= 1;
    }

}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        struct DropGuard<'a, T>(&'a mut LinkedList<T>);

        impl<'a, T> Drop for DropGuard<'a, T> {
            fn drop(&mut self) {
                while self.0.pop_front_node().is_some() {}
            }
        }

        while let Some(node) = self.pop_front_node() {
            let guard = DropGuard(self);
            drop(node);
            mem::forget(guard);
        }
    }
}

pub struct LRUKCache<K,V> {
    k: usize, 
    cap: usize,
    cache: HashMap<K,NonNull<Node<(K,V)>>>,
    cache_linked_list: LinkedList<(K,V)>,
    history_cap: usize, 
    history_cache: HashMap<K,(NonNull<Node<(K,V)>>,usize)>,
    history_linked_list: LinkedList<(K,V)>
}

impl <K,V> LRUKCache<K, V> 
where K: Hash + Eq + Clone
{
    pub fn new(k: usize, cap: usize, history_cap: usize) -> Self {
        Self { 
            k, 
            cap, 
            cache: HashMap::new(), 
            cache_linked_list: LinkedList::new(), 
            history_cap, 
            history_cache: HashMap::new(), 
            history_linked_list: LinkedList::new() }
    }
    pub fn put(&mut self, key: K, value: V) {
        self.put_history(key, value);
    }
    pub fn get(&mut self, key: &K) -> Option<&V>{
        self.hit_history(key);
        self.get_cache(key)
    }
    fn hit_history(&mut self, key: &K) {
        if let Some((_, hit_count)) = self.history_cache.get_mut(key) {
            *hit_count += 1;
            if *hit_count == self.k {
                self.history_cache.remove(key).map(|(node,_)| {
                    self.history_linked_list.unlink_node(node);
                    self.put_cache(node);
                });
            }
        }
    }
    fn put_history(&mut self, key: K, value: V) {
        if let Some((node, hit_count)) = self.history_cache.get_mut(&key) {
            unsafe {
                node.as_mut().elem = (key.clone(),value);
                *hit_count += 1;
                if *hit_count == self.k{
                    self.history_cache.remove(&key).map(|(node,_)| {
                        self.history_linked_list.unlink_node(node);
                        self.put_cache(node);
                    });
                } else {
                    self.history_linked_list.unlink_node(*node);
                    self.history_linked_list.push_front_node(*node);
                }
            }
        } else {
            let node = Node::new((key.clone(),value));
            let node = Box::leak(Box::new(node)).into();
            self.history_cache.insert(key.clone(), (node, 1));
            self.history_linked_list.push_front_node(node);
            if self.history_linked_list.len > self.history_cap {
                if let Some(delete_node) = self.history_linked_list.pop_back_node()
                {
                    self.history_cache.remove(&delete_node.elem.0);
                }
            }
        }
    }
    fn put_cache(&mut self, node: NonNull<Node<(K,V)>>){
        unsafe {
            match self.cache.remove(&node.as_ref().elem.0) {
                Some(n) => {
                    self.cache_linked_list.unlink_node(n);
                },
                None => {
                    self.cache.insert(node.as_ref().elem.0.clone(), node);
                }
            }
            self.cache_linked_list.push_front_node(node);
            if self.cache_linked_list.len > self.cap {
                if let Some(delete_node) = self.cache_linked_list.pop_back_node(){
                    self.cache.remove(&delete_node.elem.0);
                }
            }
        }
        
    }
    fn get_cache(&mut self, key: &K) -> Option<&V>{
        self.cache.get(key).map(|node| {
            self.cache_linked_list.unlink_node(*node);
            self.cache_linked_list.push_front_node(*node);
            unsafe { &node.as_ref().elem.1 }
        })
    }
}
