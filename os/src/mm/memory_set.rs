use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use bitflags::bitflags;
use lazy_static::*;
use riscv::register::satp;

use crate::config::*;
use crate::mm::map_area::MapArea;
use super::address::*;
use super::page_table::{PageTable, PageTableEntry, PTEFlags};
use crate::println;
use crate::sync::up::UPSafeCell;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
#[derive(Copy, Clone, PartialEq, Debug)]
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}


lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> = Arc::new( UPSafeCell::new(MemorySet::init_kernel()) );
}

pub struct MemorySet {
    page_table: PageTable,
    map_areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new() -> Self {
        Self {
            page_table: PageTable::new(),
            map_areas: Vec::new(),
        }
    }
    /// get page_table `level0_ppn`
    pub fn satp(&self) -> usize {
        self.page_table.to_satp()
    }
    pub fn insert_framed_areas(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }
    pub fn remove_areas(&mut self, start_vpn: VirtPageNum) {
        if let Some((idx, area)) = self
            .map_areas
            .iter_mut()
            .enumerate()
            .find(|(_, area)| area.vpn_range.get_start() == start_vpn)
        {
            area.unmap(&mut self.page_table);
            self.map_areas.remove(idx);
        }
    }
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        println!(
            "[Mem] Map area [{:#x}, {:#x})",
            map_area.vpn_range.get_start().0 << 12,
            map_area.vpn_range.get_end().0 << 12
        );
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }
        self.map_areas.push(map_area);
    }
    /// not collected by areas
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }
    pub fn init_kernel() -> Self {
        let mut memory_set = Self::new();
        // map trampoline
        println!("mapping trampoline");
        println!("[kernel] .text [{:#x}, {:#x})", TRAMPOLINE, TRAMPOLINE - 1 + PAGE_SIZE);
        memory_set.map_trampoline();
        // map kernel sections
        println!("mapping .text section");
        println!("[kernel] .text [{:#x}, {:#x})", stext as usize, etext as usize);
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        println!("mapping .rodata section");
        println!("[kernel] .rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        println!("mapping .data section");
        println!("[kernel] .data [{:#x}, {:#x})", sdata as usize, edata as usize);
        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping .bss section");
        println!("[kernel] .bss [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);
        memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping physical memory");
        println!("[kernel] physical memory [{:#x}, {:#x})", ekernel as usize, MEMORY_END);
        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping memory-mapped registers");
        println!("[kernel] memory-mapped registers for IO [{:#x}, {:#x})", SERIAL_PORT_BASE_ADDRESS, SERIAL_PORT_BASE_ADDRESS + SERIAL_PORT_MAP_SIZE);
        memory_set.push(
            MapArea::new(
                SERIAL_PORT_BASE_ADDRESS.into(),
                (SERIAL_PORT_BASE_ADDRESS + SERIAL_PORT_MAP_SIZE).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        memory_set
    }
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new();
        // map trampoline
        memory_set.map_trampoline();
        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                println!(
                    "[Mem] ELF map Virt area [{:#x}, {:#x})",
                    start_va.0 << 12,
                    end_va.0 << 12
                );
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        // map TrapContext
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }

    /// Clone a `MemorySet`
    pub fn copy_from_user(user_space: &Self) -> Self {
        let mut memory_set = Self::new();
        // map trampoline
        memory_set.map_trampoline();
        // copy data sections/trap_context/user_stack
        for area in user_space.map_areas.iter() {
            let new_area = MapArea::from_another(area);
            memory_set.push(new_area, None);
            // copy data from another space
            for vpn in area.vpn_range {
                let src_ppn = user_space.translate(vpn).unwrap().ppn();
                let dst_ppn = memory_set.translate(vpn).unwrap().ppn();
                dst_ppn
                    .get_bytes_array()
                    .copy_from_slice(src_ppn.get_bytes_array());
            }
        }
        memory_set
    }

    pub fn activate(&self) {
        let satp = self.page_table.to_satp();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    pub fn clear(&mut self) {
        self.map_areas.clear();
    }

    pub fn page_table(&self) -> &PageTable {
        &self.page_table
    }
}

pub fn activate_page_table() {
    KERNEL_SPACE.exclusive_access().activate();
}