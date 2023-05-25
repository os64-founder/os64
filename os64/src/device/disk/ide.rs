use alloc::rc::Rc;
//本文实现了DMA方式进行硬盘数据的读写
//本文用到了如下单词缩写：
//ISA: Industry Standard Architecture,工业标准体系结构。
//IDE: Integrated Drive Electronics,集成驱动器电子装置。IDE接口的硬盘，通过IDE线，连接到电脑。
//ATA: Advanced Technology Attachment,高级技术附件。
//DMA: Direct Memory Access,直接内存访问。
use bitflags::bitflags;
use lazy_static::lazy_static;
use crate::{architecture::x86_64_asm::{asm_out_byte, asm_in_byte, asm_in_u32, asm_out_u32}, serial_println, serial_print};

const SECTOR_SIZE : usize = 128;
// const MAX_DMA_SECTORS: usize = 0x1F_F000 / SECTOR_SIZE;

const IDE_COMMAND_READ      : u8 = 0x20;
const IDE_COMMAND_WRITE     : u8 = 0x30;
const IDE_COMMAND_IDENTIFY  : u8 = 0xEC;

const ISA_DATA              : u16 = 0x00;   //数据寄存器端口 
const ISA_ERROR             : u16 = 0x01;   //错误寄存器端口 
const ISA_SECTOR_COUNT      : u16 = 0x02;   //扇区计数寄存器端口
const ISA_SECTOR_INDEX      : u16 = 0x03;   //扇区号寄存器端口
const ISA_CYLINDER_LOW      : u16 = 0x04;   //柱面低字节寄存器端口   
const ISA_CYLINDER_HIGH     : u16 = 0x05;   //柱面高字节寄存器端口
const ISA_DEVICE            : u16 = 0x06;   //设备及磁头寄存器端口（各4位?）
const ISA_COMMAND           : u16 = 0x07;   //命令寄存器端口,和状态寄存器共用
const ISA_STATUS            : u16 = 0x07;   //状态寄存器端口

const PORT_IDE0_BASE        : u16 = 0x1F0;  //
const PORT_IDE1_BASE        : u16 = 0x170;  //

const PORT_IDE0_CONTROL     : u16 = 0x3F6;  //命令控制端口
const PORT_IDE1_CONTROL     : u16 = 0x376;  

bitflags! { 
    ///目录项属性
    struct IdeStatus: u8 {
        const BUSY  = 0b1000_0000;
        const READY = 0b0100_0000;
        const DF    = 0b0010_0000;
        const SEEK  = 0b0001_0000;
        const REQ   = 0b0000_1000;
        const ERROR = 0b0000_0001;
    }
}

pub struct IdeDiskDriver {
    index : u8,
    port_base : u16,
    port_control : u16,
}

lazy_static! {
    /// 两条IDE线，各两块硬盘，共4块硬盘
    pub static ref IDE_DISKS : [IdeDiskDriver; 4] = [
        IdeDiskDriver{
            index : 0,
            port_base : PORT_IDE0_BASE,
            port_control : PORT_IDE0_CONTROL,
        },
        IdeDiskDriver{
            index : 1,
            port_base : PORT_IDE0_BASE,
            port_control : PORT_IDE0_CONTROL,
        },
        IdeDiskDriver{
            index : 2,
            port_base : PORT_IDE1_BASE,
            port_control : PORT_IDE1_CONTROL,
        },
        IdeDiskDriver{
            index : 3,
            port_base : PORT_IDE1_BASE,
            port_control : PORT_IDE1_CONTROL,
        },
    ];
}

#[repr(packed)]
#[derive(Clone,Copy,Debug)]
pub struct DiskIdentifyInfo {
    ///  0   General configuration bit-significant information
    pub general_config: u16,

    ///  1   Obsolete
    pub obsolete0: u16,
    
    ///  2   Specific configuration
    pub specific_config: u16,

    ///  3   Obsolete
    pub obsolete1: u16,

    ///  4-5 Retired
    pub retired0: [u16; 2],
    
    ///  6   Obsolete
    pub obsolete2: u16,
    
    ///  7-8 Reserved for the CompactFlash Association
    pub compact_flash: [u16; 2],
    
    ///  9   Retired
    pub retired1: u16,

    ///  10-19   Serial number (20 ASCII characters)
    pub serial_number: [u8; 20],
    
    ///  20-21   Retired
    pub retired2: [u16; 2],

    ///  22  Obsolete
    pub obsolete3: u16,

    ///  23-26   Firmware revision(8 ASCII characters) 
    pub firmware_version: [u8; 8],

    ///  27-46   Model number (40 ASCII characters)
    pub model_number: [u8; 40],
    
	///	47	15:8 	80h 
	///		7:0  	00h=Reserved 
	///			01h-FFh = Maximumnumber of logical sectors that shall be transferred per DRQ data block on READ/WRITE MULTIPLE commands
    pub max_logical_transferred_per_drq: u16,

    /// 48  Trusted Computing feature set options
    pub trusted_computing_feature_set_options: u16,
    
    /// 49  Capabilities
    pub capabilities0: u16,

    /// 50  Capabilities
    pub capabilities1: u16,

    ///  51-52   Obsolete
    pub obsolete4: [u16; 2],

	///	53	15:8	Free-fall Control Sensitivity
	///		7:3 	Reserved
	///		2 	the fields reported in word 88 are valid
	///		1 	the fields reported in words (70:64) are valid
    pub report_88_70to64_valid: u16,
    
    ///  54-58   Obsolete
    pub obsolete5: [u16; 5],

	///	59	15:9	Reserved
	///		8	Multiple sector setting is valid	
	///		7:0	xxh current setting for number of logical sectors that shall be transferred per DRQ data block on READ/WRITE Multiple commands
    pub mul_sec_setting_valid: u16,

    /// 60-61   Total number of user addresssable logical sectors for 28bit CMD
    pub addressable_logical_sectors_for_28: [u16; 2], 

    ///  62   Obsolete
    pub obsolete6: u16,

	///	63	15:11	Reserved
	///		10:8=1 	Multiword DMA mode 210 is selected
	///		7:3 	Reserved
	///		2:0=1 	Multiword DMA mode 210 and below are supported
    pub mult_word_dma_select: u16,
    
	///	64	15:8	Reserved
	///		7:0	PIO mdoes supported
    pub port_mode_supported: u16,

    ///  65     Minimum Multiword DMA transfer cycle time per word
    pub min_mul_word_dma_cycle_time_per_word: u16,

    ///  66  Manufacturer`s recommended Multiword DMA transfer cycle time
    pub manufacture_recommend_mulword_dma_cycle_time: u16,

	///	67	Minimum PIO transfer cycle time without flow control
	pub min_port_cycle_time_flow_control : u16,

	///	68	Minimum PIO transfer cycle time with IORDY flow control
	pub min_port_cycle_time_ioredy_flow_control : u16,

	///	69-70	Reserved
	pub reserved1 : [ u16; 2],

	///	71-74	Reserved for the IDENTIFY PACKET DEVICE command
	pub reserved2 :[ u16; 4],

	///	75	Queue depth
	pub queue_depth : u16,

	///	76	Serial ATA Capabilities 
	pub sata_capabilities : u16,

	///	77	Reserved for Serial ATA 
	pub reserved3 : u16,

	///	78	Serial ATA features Supported 
	pub sata_features_supported : u16,

	///	79	Serial ATA features enabled
	pub sata_features_enabled : u16,

	///	80	Major Version number
	pub major_version : u16,

	///	81	Minor version number
	pub minor_version : u16,

	///	82	Commands and feature sets supported
	pub cmd_feature_sets_supported0 : u16,

	///	83	Commands and feature sets supported	
	pub cmd_feature_sets_supported1 : u16,

	///	84	Commands and feature sets supported
	pub cmd_feature_sets_supported2 : u16,

	///	85	Commands and feature sets supported or enabled
	pub cmd_feature_sets_supported3 : u16,

	///	86	Commands and feature sets supported or enabled
	pub cmd_feature_sets_supported4 : u16,

	///	87	Commands and feature sets supported or enabled
	pub cmd_feature_sets_supported5 : u16,

	///	88	15 	Reserved 
	///		14:8=1 	Ultra DMA mode 6543210 is selected 
	///		7 	Reserved 
	///		6:0=1 	Ultra DMA mode 6543210 and below are suported
	pub ultra_dma_modes : u16,

	///	89	Time required for Normal Erase mode SECURITY ERASE UNIT command
	pub time_required_erase_cmd : u16,

	///	90	Time required for an Enhanced Erase mode SECURITY ERASE UNIT command
	pub time_required_enhanced_cmd : u16,

	///	91	Current APM level value
	pub current_apm_level_value : u16,

	///	92	Master Password Identifier
	pub master_password_identifier : u16,

	///	93	Hardware resset result.The contents of bits (12:0) of this word shall change only during the execution of a hardware reset.
	pub hard_ware_reset_result : u16,

	///	94	Current AAM value 
	///		15:8 	Vendor’s recommended AAM value 
	///		7:0 	Current AAM value
	pub current_aam_value : u16,

	///	95	Stream Minimum Request Size
	pub stream_min_request_size : u16,

	///	96	Streaming Transger Time-DMA 
	pub streaming_transger_time_dma : u16,

	///	97	Streaming Access Latency-DMA and PIO
	pub streaming_access_latency_dma_pio : u16,

	///	98-99	Streaming Performance Granularity (DWord)
	pub streaming_performance_granularity : [u16; 2],

	///	100-103	Total Number of User Addressable Logical Sectors for 48-bit commands (QWord)
	pub total_user_lba_for_48_address_feature_set : u64,

	///	104	Streaming Transger Time-PIO
	pub streaming_transfer_time_pio : u16,

	///	105	Reserved
	pub reserved4 : u16,

	///	106	Physical Sector size/Logical Sector Size
	pub physical_logical_sector_size : u16,

	///	107	Inter-seek delay for ISO-7779 acoustic testing in microseconds
	pub inter_seek_delay : u16,

	///	108-111	World wide name	
	pub world_wide_name : [u16; 4],

	///	112-115	Reserved
	pub reserved5 : [u16; 4],

	///	116	Reserved for TLC
	pub reserved6 : u16,

	///	117-118	Logical sector size (DWord)
	pub words_per_logical_sector : [u16; 2],

	///	119	Commands and feature sets supported (Continued from words 84:82)
	pub cmd_feature_supported : u16,

	///	120	Commands and feature sets supported or enabled (Continued from words 87:85)
	pub cmd_feature_supported_enabled : u16,

	///	121-126	Reserved for expanded supported and enabled settings
	pub reserved7 : [u16; 6],

	///	127	Obsolete
	pub obsolete7 : u16,

	///	128	Security status
	pub security_status : u16,

	///	129-159	Vendor specific
	pub vendor_specific : [u16; 31],

	///	160	CFA power mode
	pub cfa_power_mode : u16,

	///	161-167	Reserved for the CompactFlash Association
	pub reserved8 : [u16; 7],

	///	168	Device Nominal Form Factor
	pub dev_from_factor : u16,

	///	169-175	Reserved
	pub reserved9 : [u16; 7],

	///	176-205	Current media serial number (ATA string)
	pub current_media_serial_number : [u16; 30],

	///	206	SCT Command Transport
	pub sct_cmd_transport : u16,

	///	207-208	Reserved for CE-ATA
	pub reserved10 : [u16; 2],

	///	209	Alignment of logical blocks within a physical block
	pub alignment_logical_blocks_within_a_physical_block : u16,

	///	210-211	Write-Read-Verify Sector Count Mode 3 (DWord)
	pub write_read_verify_sector_count_mode_3 : [u16; 2],

	///	212-213	Write-Read-Verify Sector Count Mode 2 (DWord)
	pub write_read_verify_sector_count_mode_2 : [u16; 2],

	///	214	NV Cache Capabilities
	pub nv_cache_capabilities : u16,

	///	215-216	NV Cache Size in Logical Blocks (DWord)
	pub nv_cache_size : [u16; 2],

	///	217	Nominal media rotation rate
	pub nominal_media_rotation_rate : u16,

	///	218	Reserved
	pub reserved11 : u16,

	///	219	NV Cache Options
	pub nv_cache_options : u16,

	///	220	Write-Read-Verify feature set current mode
	pub write_read_verify_feature_set_current_mode : u16,

	///	221	Reserved
	pub reserved12 : u16,

	///	222	Transport major version number. 
	///		0000h or ffffh = device does not report version
	pub transport_major_version_number : u16,

	///	223	Transport Minor version number
	pub transport_minor_version_number : u16,

	///	224-233	Reserved for CE-ATA
	pub reserved13 : [u16; 10],

	///	234	Minimum number of 512-byte data blocks per DOWNLOAD MICROCODE command for mode 03h
	pub mini_blocks_per_cmd : u16,

	///	235	Maximum number of 512-byte data blocks per DOWNLOAD MICROCODE command for mode 03h
	pub max_blocks_per_cmd : u16,

	///	236-254	Reserved
	pub reserved14 : [u16; 19],

	///	255	Integrity word
	///		15:8	Checksum
	///		7:0	Checksum Validity Indicator
	pub integrity_word : u16,
}

impl IdeDiskDriver {
    fn wait(&self) {
        while unsafe { asm_in_byte(self.port_base + ISA_STATUS) } & IdeStatus::BUSY.bits != 0 {}
    }

    fn wait_error(&self) -> bool {
        self.wait();
        let status = unsafe { asm_in_byte(self.port_base + ISA_STATUS) };
        status & (IdeStatus::DF.bits | IdeStatus::ERROR.bits) != 0
    }

    fn select(&self, sector: u64, count: u8) {
        assert_ne!(count, 0);
        self.wait();
        unsafe {
            // generate interrupt
            asm_out_byte(self.port_control, 0);
            asm_out_byte(self.port_base + ISA_SECTOR_COUNT, count);
            asm_out_byte(self.port_base + ISA_SECTOR_INDEX, (sector & 0xFF) as u8);
            asm_out_byte(self.port_base + ISA_CYLINDER_LOW, ((sector >> 8) & 0xFF) as u8);
            asm_out_byte(self.port_base + ISA_CYLINDER_HIGH, ((sector >> 16) & 0xFF) as u8);
            asm_out_byte(self.port_base + ISA_DEVICE,
                0xE0 | ((self.index & 1) << 4) | (((sector >> 24) & 0xF) as u8),
            );
        }
    }

    pub fn init(&self) -> Result<DiskIdentifyInfo,()> {
        self.wait();
        unsafe {
            // step1: select drive
            asm_out_byte(self.port_base + ISA_DEVICE, (0xE0 | ((self.index & 1) << 4)) as u8);
            self.wait();

            // step2: send ATA identify command
            asm_out_byte(self.port_base + ISA_COMMAND, IDE_COMMAND_IDENTIFY);
            self.wait();

            // step3: polling
            if asm_in_byte(self.port_base + ISA_STATUS) == 0 || self.wait_error() {
                return Err(());
            }

            //读取磁盘信息
            let mut data = [0; SECTOR_SIZE];
            asm_in_u32(self.port_base + ISA_DATA, data.as_mut_ptr(), SECTOR_SIZE);
            let disk_info = *(data.as_mut_ptr() as *mut DiskIdentifyInfo);
            // serial_println!("{:?}", disk_info);
            print_u8_arrays("serial_number = ",disk_info.serial_number.as_ptr(),20);
            print_u8_arrays("firmware_version = ",disk_info.firmware_version.as_ptr(),8);
            print_u8_arrays("model_number = ",disk_info.model_number.as_ptr(),40);
            let total_sector = disk_info.total_user_lba_for_48_address_feature_set;
            let total_kb = total_sector / 2;
            if total_kb < 1024  {
                serial_println!("total_size = {} KB",total_kb);
            } else {
                let total_mb = total_kb as f64 / 1024.0;
                if total_mb < 1024.0 {
                    serial_println!("total_size = {} MB",total_mb);
                } else {
                    let total_gb = total_mb as f64 / 1024.0;
                    serial_println!("total_size = {} GB",total_gb);
                }
            }
            Ok(disk_info)
        }
    }

    /// Read ATA DMA. Block size = 512 bytes.
    pub fn read(&self, sector: u64, count: usize, data: &mut [u32]) -> Result<(), ()> {
        assert_eq!(data.len(), count * SECTOR_SIZE);
        self.wait();
        unsafe {
            self.select(sector, count as u8);
            asm_out_byte(self.port_base + ISA_COMMAND, IDE_COMMAND_READ);
            for i in 0..count {
                let ptr = &mut data[(i as usize) * SECTOR_SIZE];
                if self.wait_error() {
                    return Err(());
                }
                asm_in_u32(self.port_base + ISA_DATA, ptr, SECTOR_SIZE);
            }
            for i in data {
                serial_print!("{:04x} ",i);
            }        
        }
        Ok(())
    }

    /// Write ATA DMA. Block size = 512 bytes.
    pub fn write(&self, sector: u64, count: usize, data: &[u32]) -> Result<(), ()> {
        assert_eq!(data.len(), count * SECTOR_SIZE);
        self.wait();
        unsafe {
            self.select(sector, count as u8);
            asm_out_byte(self.port_base + ISA_COMMAND, IDE_COMMAND_WRITE);
            for i in 0..count {
                let ptr = &data[(i as usize) * SECTOR_SIZE];
                if self.wait_error() {
                    return Err(());
                }
                asm_out_u32(self.port_base + ISA_DATA, ptr, SECTOR_SIZE);
            }
        }
        Ok(())
    }
}

pub fn ide_handler(ide_index : usize) {
}

pub fn print_u8_arrays(title : &str, string : *const u8, size : isize ) {
    serial_print!("{}",title);
    unsafe {
        for i in 0..size {
            serial_print!("{}",*(string.offset(i)) as char);
        }
    }
    serial_print!("\n");
}
