//IDE ATA 

const PORT_DISK0_DATA            : u16 = 0x1F0;  //数据寄存器端口
const PORT_DISK0_ERROR           : u16 = 0x1F1;  //错误寄存器端口 
const PORT_DISK0_SECTOR_COUNT    : u16 = 0x1F2;  //扇区计数寄存器端口
const PORT_DISK0_SECTOR_NUMBER   : u16 = 0x1F3;  //扇区号寄存器端口
const PORT_DISK0_CYLINDER_LOW    : u16 = 0x1F4;  //柱面低字节寄存器端口
const PORT_DISK0_CYLINDER_HIGH   : u16 = 0x1F5;  //柱面高字节寄存器端口
const PORT_DISK0_DEVICE          : u16 = 0x1F6;  //设备/磁头寄存器端口
const PORT_DISK0_STATUS_COMMAND  : u16 = 0x1F7;  //命令寄存器端口,和状态寄存器共用

const PORT_DISK1_DATA            : u16 = 0x170;  //数据寄存器端口
const PORT_DISK1_ERROR           : u16 = 0x171;  //错误寄存器端口 
const PORT_DISK1_SECTOR_COUNT    : u16 = 0x172;  //扇区计数寄存器端口
const PORT_DISK1_SECTOR_NUMBER   : u16 = 0x173;  //扇区号寄存器端口
const PORT_DISK1_CYLINDER_LOW    : u16 = 0x174;  //柱面低字节寄存器端口
const PORT_DISK1_CYLINDER_HIGH   : u16 = 0x175;  //柱面高字节寄存器端口
const PORT_DISK1_DEVICE          : u16 = 0x176;  //设备/磁头寄存器端口
const PORT_DISK1_STATUS_COMMAND  : u16 = 0x177;  //命令寄存器端口,和状态寄存器共用

const PORT_DISK0_CONTROL         : u16 = 0x3F6;  //命令控制端口
const PORT_DISK1_CONTROL         : u16 = 0x376;  //命令控制端口

const DISK_STATUS_BUSY          : u8 = 1<<7;    //命令寄存器端口,和状态寄存器共用
const DISK_STATUS_READY         : u8 = 1<<6;    //命令寄存器端口,和状态寄存器共用
const DISK_STATUS_SEEK          : u8 = 1<<4;    //命令寄存器端口,和状态寄存器共用
const DISK_STATUS_REQ           : u8 = 1<<3;    //命令寄存器端口,和状态寄存器共用
const DISK_STATUS_ERROR         : u8 = 1<<0;    //命令寄存器端口,和状态寄存器共用

#[repr(packed)]
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
    pub serial_number: [u16; 10],
    
    ///  20-21   Retired
    pub retired2: [u16; 2],

    ///  22  Obsolete
    pub obsolete3: u16,

    ///  23-26   Firmware revision(8 ASCII characters) 
    pub firmware_version: [u16; 4],

    ///  27-46   Model number (40 ASCII characters)
    pub model_number: [u16; 20],
    
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
    pub PORT_mode_supported: u16,

    ///  65     Minimum Multiword DMA transfer cycle time per word
    pub min_mul_word_dma_cycle_time_per_word: u16,

    ///  66  Manufacturer`s recommended Multiword DMA transfer cycle time
    pub manufacture_recommend_mulword_dma_cycle_time: u16,

	///	67	Minimum PIO transfer cycle time without flow control
	pub min_PORT_cycle_time_flow_control : u16,

	///	68	Minimum PIO transfer cycle time with IORDY flow control
	pub min_PORT_cycle_time_ioredy_flow_control : u16,

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
	pub total_user_lba_for_48_address_feature_set : [u16; 4],

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

// hw_int_controller disk_int_controller = 
// {
// 	.enable = IOAPIC_enable,
// 	.disable = IOAPIC_disable,
// 	.install = IOAPIC_install,
// 	.uninstall = IOAPIC_uninstall,
// 	.ack = IOAPIC_edge_ack,
// };

// void disk_handler(unsigned long nr, unsigned long parameter, struct pt_regs * regs)
// {
// 	int i = 0;
// 	unsigned char a[512];
// 	port_insw(PORT_DISK1_DATA,&a,256);
// 	color_printk(ORANGE,WHITE,"Read One Sector Finished:%02x\n",io_in8(PORT_DISK1_STATUS_CMD));
// 	for(i = 0;i<512;i++)
// 		color_printk(ORANGE,WHITE,"%02x ",a[i]);
// }


use crate::{architecture::x86_64_asm::{asm_out_byte, asm_in_byte}, device::serial, serial_println, parallel::apic::*};

// bitfield!{
//     struct Date(MSB0 [u8]);
//     u32;
//     get_day, set_day: 4, 0;
//     get_month, set_month: 8, 5;
//     get_year, set_year: 23, 9;
// }

// fn main() {
//     let mut date = Date([0, 0, 0]);

//     date.set_day(7);
//     date.set_month(1);
//     date.set_year(2020);

//     assert_eq!(date.get_day(), 7);
//     assert_eq!(date.get_month(), 1);
//     assert_eq!(date.get_year(), 2020);
// }

pub fn disk_init() {
	let mut entry = IO_APIC_RET_entry([
        0x2f,
        APIC_ICR_IOAPIC_FIXED,
        ICR_IOAPIC_DELV_PHYSICAL,
        APIC_ICR_IOAPIC_IDLE,
        APIC_IOAPIC_POLARITY_HIGH,
        APIC_IOAPIC_IRR_RESET,
        APIC_ICR_IOAPIC_EDGE,
        APIC_ICR_IOAPIC_MASKED,
        0,
        0,
        0,
    ]);

	// register_irq(0x2f, &entry , &disk_handler, 0, &disk_int_controller, "disk1");

	unsafe { 
        asm_out_byte(PORT_DISK1_CONTROL,0);
	    while (asm_in_byte(PORT_DISK1_STATUS_COMMAND) & DISK_STATUS_BUSY) != 0 {}
	    serial_println!("Read One Sector Starting:{}\n",asm_in_byte(PORT_DISK1_STATUS_COMMAND));

        asm_out_byte(PORT_DISK1_DEVICE,0xe0);
        asm_out_byte(PORT_DISK1_ERROR,0);
        asm_out_byte(PORT_DISK1_SECTOR_COUNT,1);
        asm_out_byte(PORT_DISK1_SECTOR_NUMBER,0);
        asm_out_byte(PORT_DISK1_CYLINDER_LOW,0);
        asm_out_byte(PORT_DISK1_CYLINDER_HIGH,0);

    	while (asm_in_byte(PORT_DISK1_STATUS_COMMAND) & DISK_STATUS_READY) == 0 {}
    	serial_println!("Send CMD:{}",asm_in_byte(PORT_DISK1_STATUS_COMMAND));
        //read
        asm_out_byte(PORT_DISK1_STATUS_COMMAND,0x20);	
    };
	
}

pub fn disk_exit() {
	// unregister_irq(0x2f);
}

pub fn disk_handler(nr : u64, parameter : u64, regs : pt_regs) {
	// let int i = 0;
	// unsigned char a[512];
	// port_insw(PORT_DISK1_DATA,&a,256);
	// color_printk(ORANGE,WHITE,"Read One Sector Finished:%02x\n",io_in8(PORT_DISK1_STATUS_CMD));
	// for(i = 0;i<512;i++)
	// 	color_printk(ORANGE,WHITE,"%02x ",a[i]);
}

// fn read_sectors(&mut self, count: u32, start_sector: u32) {
//     //等待设备准备好
//     self.wait_ready();
    
//     //写入扇区数和起始扇区号
//     unsafe {
//         port_write(PORT_SECTOR_COUNT, count as u16);
//         port_write(PORT_SECTOR_NUMBER, start_sector as u16);
//     }
    
//     //发送读扇区命令
//     unsafe { port_write(PORT_COMMAND, 0x20) };  

//     //读取数据和状态
//     let mut buf = vec![0; 512 * count as usize];
//     self.read_buffer(buf.as_mut_slice());
//     let status = unsafe { port_read(PORT_STATUS) };
// }

// unsafe fn port_read(port: u16) -> u8 {
//     *((port as u32) as *mut u8)
// } 

// unsafe fn port_write(port: u16, data: u8) {
//     *((port as u32) as *mut u8) = data;
// }
