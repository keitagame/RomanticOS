use x86_64::{
    structures::paging::{
        PageTable, OffsetPageTable, Page, PhysFrame, Mapper, Size4KiB,
        FrameAllocator, PageTableFlags as Flags,
    },
    VirtAddr, PhysAddr,
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use linked_list_allocator::LockedHeap;
use spin::Mutex;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

static MEMORY_MANAGER: Mutex<Option<MemoryManager>> = Mutex::new(None);

pub struct MemoryManager {
    mapper: OffsetPageTable<'static>,
    frame_allocator: BootInfoFrameAllocator,
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub fn init(boot_info: &'static bootloader::BootInfo) {
    const PHYS_OFFSET: u64 = 0xffff_8000_0000_0000;
    let phys_mem_offset = VirtAddr::new(PHYS_OFFSET);

    //let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { init_mapper(phys_mem_offset) };
    let frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    let mut manager = MemoryManager {
        mapper,
        frame_allocator,
    };

    *MEMORY_MANAGER.lock() = Some(manager);
}

unsafe fn init_mapper(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub fn init_heap() -> Result<(), &'static str> {
    let mut manager = MEMORY_MANAGER.lock();
    let manager = manager.as_mut().ok_or("Memory manager not initialized")?;

    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = manager.frame_allocator
            .allocate_frame()
            .ok_or("out of memory")?;
        let flags = Flags::PRESENT | Flags::WRITABLE;
        unsafe {
            manager.mapper.map_to(page, frame, flags, &mut manager.frame_allocator)
                .map_err(|_| "map_to failed")?
                .flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
        //ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub fn allocate_pages(count: usize) -> Option<VirtAddr> {
    let mut manager = MEMORY_MANAGER.lock();
    let manager = manager.as_mut()?;

    // 仮想アドレス空間から連続したページを見つける
    let start_page = find_free_pages(count)?;
    
    for i in 0..count {
        let page = start_page + i as u64;
        let frame = manager.frame_allocator.allocate_frame()?;
        let flags = Flags::PRESENT | Flags::WRITABLE | Flags::USER_ACCESSIBLE;
        
        unsafe {
            manager.mapper
                .map_to(page, frame, flags, &mut manager.frame_allocator)
                .ok()?
                .flush();
        }
    }

    Some(start_page.start_address())
}

fn find_free_pages(count: usize) -> Option<Page> {
    // 簡易実装: ユーザー空間の先頭から検索
    // 実際の実装ではビットマップなどで管理
    const USER_SPACE_START: u64 = 0x0000_4000_0000_0000;
    let start_addr = VirtAddr::new(USER_SPACE_START);
    Some(Page::containing_address(start_addr))
}

pub fn deallocate_pages(addr: VirtAddr, count: usize) {
    let mut manager = MEMORY_MANAGER.lock();
    if let Some(manager) = manager.as_mut() {
        use x86_64::structures::paging::Size4KiB;

        let start_page: Page<Size4KiB> = Page::containing_address(addr);

        //let start_page = Page::containing_address(addr);
        
        for i in 0..count {
            let page = start_page + i as u64;
            if let Ok((_, flush)) = manager.mapper.unmap(page) {
                flush.flush();
            }
        }
    }
}
