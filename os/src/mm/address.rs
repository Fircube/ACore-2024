use crate::config::*;
use crate::mm::page_table::PageTableEntry;

// Type definition
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct VirtPageNum(pub usize);

impl PhysAddr {
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PhysPageNum {
        if self.0 == 0 {
            PhysPageNum(0)
        } else {
            PhysPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl PhysPageNum {
    // 页表项定长数组的可变引用，代表多级页表中的一个节点
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = (*self).into();
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512)
        }
    }
    // 字节数组的可变引用，可以以字节为粒度对物理页帧上的数据进行访问
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096)
        }
    }
    // 泛型函数，可以获取一个恰好放在一个物理页帧开头的类型为 T 的数据的可变引用。
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        unsafe {
            (pa.0 as *mut T).as_mut().unwrap()
        }
    }

    // `'static` 为了绕过 Rust 编译器的借用检查，可以像一个正常的可变引用一样直接访问。
}

impl VirtAddr {
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> VirtPageNum {
        if self.0 == 0 {
            VirtPageNum(0)
        } else {
            VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}


impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}

// Type conversion from usize to addr/PageNum
impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PA_WIDTH_SV39) - 1))
    }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VA_WIDTH_SV39) - 1))
    }
}

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VPN_WIDTH_SV39) - 1))
    }
}

// Type conversion from addr/PageNum to usize
impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}

impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
            v.0 | (!((1 << VA_WIDTH_SV39) - 1))
        } else {
            v.0
        }
    }
}

impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}

// Type conversion between PhysAddr and PhysPageNum
impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

// Type conversion between VirtAddr and VirtPageNum
impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}


// pub trait StepByOne {
//     fn step(&mut self);
// }
// impl StepByOne for VirtPageNum {
//     fn step(&mut self) {
//         self.0 += 1;
//     }
// }
//
// #[derive(Copy, Clone)]
// /// a simple range structure for type T
// pub struct SimpleRange<T>
//     where
//         T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
// {
//     l: T,
//     r: T,
// }
// impl<T> SimpleRange<T>
//     where
//         T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
// {
//     pub fn new(start: T, end: T) -> Self {
//         assert!(start <= end, "start {:?} > end {:?}!", start, end);
//         Self { l: start, r: end }
//     }
//     pub fn get_start(&self) -> T {
//         self.l
//     }
//     pub fn get_end(&self) -> T {
//         self.r
//     }
// }
// impl<T> IntoIterator for SimpleRange<T>
//     where
//         T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
// {
//     type Item = T;
//     type IntoIter = SimpleRangeIterator<T>;
//     fn into_iter(self) -> Self::IntoIter {
//         SimpleRangeIterator::new(self.l, self.r)
//     }
// }
// /// iterator for the simple range structure
// pub struct SimpleRangeIterator<T>
//     where
//         T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
// {
//     current: T,
//     end: T,
// }
// impl<T> SimpleRangeIterator<T>
//     where
//         T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
// {
//     pub fn new(l: T, r: T) -> Self {
//         Self { current: l, end: r }
//     }
// }
// impl<T> Iterator for SimpleRangeIterator<T>
//     where
//         T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
// {
//     type Item = T;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.current == self.end {
//             None
//         } else {
//             let t = self.current;
//             self.current.step();
//             Some(t)
//         }
//     }
// }
//
// /// a simple range structure for virtual page number
// pub type VPNRange = SimpleRange<VirtPageNum>;