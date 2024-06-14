#[derive(Copy, Clone)]
pub struct LinkedList {
    head: *mut usize,
}

impl LinkedList {
    // Create a new LinkedList
    pub const fn new() -> LinkedList {
        LinkedList {
            head: core::ptr::null_mut(),
        }
    }

    // Return `true` if the list is empty
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    // Push `node` to the front of the list
    pub unsafe fn push(&mut self, node: *mut usize) {
        *node = self.head as usize;
        self.head = node;
    }

    // Pop the first node in the list
    pub fn pop(&mut self) -> Option<*mut usize> {
        if self.is_empty() {
            None
        } else {
            let node = self.head;
            self.head = unsafe { *node as *mut usize };
            Some(node)
        }
    }

    pub fn iter(&self) -> LinkedListIter {
        LinkedListIter {
            curr: self.head,
            linked_list: self,
        }
    }

    pub fn iter_mut(&mut self) -> LinkedListMutIter {
        LinkedListMutIter {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            curr: self.head,
            linked_list: self,
        }
    }
}

// Represent a mutable node in `LinkedList`
pub struct ListNode {
    prev: *mut usize,
    curr: *mut usize,
}

impl ListNode {
    // Remove the node from the list
    pub fn pop(self) -> *mut usize {
        unsafe {
            *(self.prev) = *(self.curr);
        }
        self.curr
    }

    // Returns the pointed address
    pub fn ptr(&self) -> *mut usize {
        self.curr
    }

    pub fn value(&self) -> usize {
        unsafe { *self.curr }
    }
}

pub struct LinkedListIter<'a> {
    curr: *mut usize,
    linked_list: &'a LinkedList,
}

impl<'a> Iterator for LinkedListIter<'a> {
    type Item = *mut usize;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.curr.is_null() {
            let result = self.curr;
            self.curr = unsafe { *self.curr } as *mut usize;
            Some(result)
        } else {
            None
        }
    }
}

pub struct LinkedListMutIter<'a> {
    prev: *mut usize,
    curr: *mut usize,
    linked_list: &'a mut LinkedList,
}

impl<'a> Iterator for LinkedListMutIter<'a> {
    type Item = ListNode;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.curr.is_null() {
            let result = ListNode {
                prev: self.prev,
                curr: self.curr,
            };
            self.prev = self.curr;
            self.curr = unsafe { *self.curr } as *mut usize;
            Some(result)
        } else {
            None
        }
    }
}
