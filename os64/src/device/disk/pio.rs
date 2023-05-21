const PIO_DATA: u16 = 0x1F0;     //数据寄存器端口
const PIO_ERROR: u16 = 0x1F1;    //错误寄存器端口 
const PIO_SECTOR_COUNT: u16 = 0x1F2;   //扇区计数寄存器端口
const PIO_SECTOR_NUMBER: u16 = 0x1F3;  //扇区号寄存器端口
const PIO_CYLINDER_LOW: u16 = 0x1F4;   //柱面低字节寄存器端口
const PIO_CYLINDER_HIGH: u16 = 0x1F5;  //柱面高字节寄存器端口
const PIO_DEVICE_HEAD: u16 = 0x1F6;    //设备/磁头寄存器端口
const PIO_STATUS: u16 = 0x1F7;         //状态寄存器端口  
const PIO_COMMAND: u16 = 0x1F7;        //命令寄存器端口,和状态寄存器共用

// const PIO_DATA: u16 = 0x170;     //数据寄存器端口
// const PIO_ERROR: u16 = 0x171;    //错误寄存器端口 
// const PIO_SECTOR_COUNT: u16 = 0x172;   //扇区计数寄存器端口
// const PIO_SECTOR_NUMBER: u16 = 0x173;  //扇区号寄存器端口
// const PIO_CYLINDER_LOW: u16 = 0x174;   //柱面低字节寄存器端口
// const PIO_CYLINDER_HIGH: u16 = 0x175;  //柱面高字节寄存器端口
// const PIO_DEVICE_HEAD: u16 = 0x176;    //设备/磁头寄存器端口
// const PIO_STATUS: u16 = 0x177;         //状态寄存器端口  
// const PIO_COMMAND: u16 = 0x177;        //命令寄存器端口,和状态寄存器共用

const DISK_STATUS_BUSY: u8 = 1<<7;        //命令寄存器端口,和状态寄存器共用
const DISK_STATUS_READY: u8 = 1<<6;        //命令寄存器端口,和状态寄存器共用
const DISK_STATUS_SEEK: u8 = 1<<4;        //命令寄存器端口,和状态寄存器共用
const DISK_STATUS_REQ: u8 = 1<<3;        //命令寄存器端口,和状态寄存器共用
const DISK_STATUS_ERROR: u8 = 1<<0;        //命令寄存器端口,和状态寄存器共用



fn read_sectors(&mut self, count: u32, start_sector: u32) {
    //等待设备准备好
    self.wait_ready();
    
    //写入扇区数和起始扇区号
    unsafe {
        port_write(PIO_SECTOR_COUNT, count as u16);
        port_write(PIO_SECTOR_NUMBER, start_sector as u16);
    }
    
    //发送读扇区命令
    unsafe { port_write(PIO_COMMAND, 0x20) };  

    //读取数据和状态
    let mut buf = vec![0; 512 * count as usize];
    self.read_buffer(buf.as_mut_slice());
    let status = unsafe { port_read(PIO_STATUS) };
}

unsafe fn port_read(port: u16) -> u8 {
    *((port as u32) as *mut u8)
} 

unsafe fn port_write(port: u16, data: u8) {
    *((port as u32) as *mut u8) = data;
}

#[repr(packed)]
pub struct DiskIdentifyInfo {
    pub general_config: u16,           //  0   General configuration bit-significant information      
    pub obsolete0: u16,               //  1   Obsolete
    pub specific_config: u16,         //  2   Specific configuration     
    pub obsolete1: u16,               //  3   Obsolete
    pub retired0: [u16; 2],          //  4-5 Retired
    pub obsolete2: u16,               //  6   Obsolete
    pub compact_flash: [u16; 2],     //  7-8 Reserved for the CompactFlash Association     
    pub retired1: u16,                //  9   Retired
    pub serial_number: [u16; 10],    //  10-19   Serial number (20 ASCII characters)
    pub retired2: [u16; 2],          //  20-21   Retired
    pub obsolete3: u16,               //  22  Obsolete
    pub firmware_version: [u16; 4],   //  23-26   Firmware revision(8 ASCII characters) 
    pub model_number: [u16; 20],     //  27-46   Model number (40 ASCII characters)
    pub max_logical_transferred_per_drq: u16, //  47  
    pub trusted_computing_feature_set_options: u16,   //  48  
    pub capabilities0: u16,          //  49    
    pub capabilities1: u16,          //  50    
    pub obsolete4: [u16; 2],         //  51-52   Obsolete
    pub report_88_70to64_valid: u16, //  53     
    pub obsolete5: [u16; 5],        //  54-58   Obsolete
    pub mul_sec_setting_valid: u16,  //  59     
    pub addressable_logical_sectors_for_28: [u16; 2], // 60-61 
    pub obsolete6: u16,              //  62   Obsolete
    pub mult_word_dma_select: u16,   //  63     
    pub pio_mode_supported: u16,     //  64     
    pub min_mul_word_dma_cycle_time_per_word: u16,   //  65
    pub manufacture_recommend_mulword_dma_cycle_time: u16, //  66  Manufacturer`s recommended Multiword DMA transfer cycle time
    // //  67  Minimum PIO transfer cycle time without flow control
    // unsigned short Min_PIO_cycle_time_Flow_Control;
    // //  68  Minimum PIO transfer cycle time with IORDY flow control
    // unsigned short Min_PIO_cycle_time_IOREDY_Flow_Control;
    // //  69-70   Reserved
    // unsigned short Reserved1[2];
    // //  71-74   Reserved for the IDENTIFY PACKET DEVICE command
    // unsigned short Reserved2[4];
    // //  75  Queue depth
    // unsigned short Queue_depth;
    // //  76  Serial ATA Capabilities
    // unsigned short SATA_Capabilities;
    // //  77  Reserved for Serial ATA
    // unsigned short Reserved3;
    // //  78  Serial ATA features Supported
    // unsigned short SATA_features_Supported;
    // //  79  Serial ATA features enabled
    // unsigned short SATA_features_enabled;
    // //  80  Major Version number
    // unsigned short Major_Version;
    // //  81  Minor version number
    // unsigned short Minor_Version;
    // //  82  Commands and feature sets supported
    // unsigned short Cmd_feature_sets_supported0;
    // //  83  Commands and feature sets supported
    // unsigned short Cmd_feature_sets_supported1;
    // //  84  Commands and feature sets supported
    // unsigned short Cmd_feature_sets_supported2;
    // //  85  Commands and feature sets supported or enabled
    // unsigned short Cmd_feature_sets_supported3;
    // //  86  Commands and feature sets supported or enabled
    // unsigned short Cmd_feature_sets_supported4;
    // //  87  Commands and feature sets supported or enabled
    // unsigned short Cmd_feature_sets_supported5;
    // //  88  15  Reserved
    // //      14:8=1  Ultra DMA mode 6543210 is selected
    // //      7   Reserved
    // //      6:0=1   Ultra DMA mode 6543210 and below are suported
    // unsigned short Ultra_DMA_modes;
    // //  89  Time required for Normal Erase mode SECURITY ERASE UNIT command
    // unsigned short Time_required_Erase_CMD;
    // //  90  Time required for an Enhanced Erase mode SECURITY ERASE UNIT command
    // unsigned short Time_required_Enhanced_CMD;
    // //  91  Current APM level value
    // unsigned short Current_APM_level_Value;
    // //  92  Master Password Identifier
    // unsigned short Master_Password_Identifier;
    // //  93  Hardware resset result.The contents of bits (12:0) of this word shall change only during the execution of a hardware reset.
    // unsigned short HardWare_Reset_Result;
    // //  94  Current AAM value
    // //      15:8    Vendor’s recommended AAM value
    // //      7:0     Current AAM value
    // unsigned short Current_AAM_value;
    // //  95  Stream Minimum Request Size
    // unsigned short Stream_Min_Request_Size;
    // //  96  Streaming Transger Time-DMA
    // unsigned short Streaming_Transger_time_DMA;
    // //  97  Streaming Access Latency-DMA and PIO
    // unsigned short Streaming_Access_Latency_DMA_PIO;
    // //  98-99   Streaming Performance Granularity (DWord)
    // unsigned short Streaming_Performance_Granularity[2];
    // //  100-103 Total Number of User Addressable Logical Sectors for 48-bit commands (QWord)
    // unsigned short Total_user_LBA_for_48_Address_Feature_set[4];
    // //  104 Streaming Transger Time-PIO
    // unsigned short Streaming_Transfer_Time_PIO;
    // //  105 Reserved
    // unsigned short Reserved4;
    // //  106 Physical Sector size/Logical Sector Size
    // unsigned short Physical_Logical_Sector_Size;
    // //  107 Inter-seek delay for ISO-7779 acoustic testing in microseconds
    // unsigned short Inter_seek_delay;
    // //  108-111 World wide name
    // unsigned short World_wide_name[4];
    // //  112-115 Reserved
    // unsigned short Reserved5[4];
    // //  116 Reserved for TLC
    // unsigned short Reserved6;
    // //  117-118 Logical sector size (DWord)
    // unsigned short Words_per_Logical_Sector[2];
    // //  119 Commands and feature sets supported (Continued from words 84:82)
    // unsigned short CMD_feature_Supported;
    // //  120 Commands and feature sets supported or enabled (Continued from words 87:85)
    // unsigned short CMD_feature_Supported_enabled;
    // //  121-126 Reserved for expanded supported and enabled settings
    // unsigned short Reserved7[6];
    // //  127 Obsolete
    // unsigned short Obsolete7;
    // //  128 Security status
    // unsigned short Security_Status;
    // //  129-159 Vendor specific
    // unsigned short Vendor_Specific[31];
    // //  160 CFA power mode
    // unsigned short CFA_Power_mode;
    // //  161-167 Reserved for the CompactFlash Association
    // unsigned short Reserved8[7];
    // //  168 Device Nominal Form Factor
    // unsigned short Dev_from_Factor;
    // //  169-175 Reserved
    // unsigned short Reserved9[7];
    // //  176-205 Current media serial number (ATA string)
    // unsigned short Current_Media_Serial_Number[30];
    // //  206 SCT Command Transport
    // unsigned short SCT_Cmd_Transport;
    // //  207-208 Reserved for CE-ATA
    // unsigned short Reserved10[2];
    // //  209 Alignment of logical blocks within a physical block
    // unsigned short Alignment_Logical_blocks_within_a_physical_block;
    // //  210-211 Write-Read-Verify Sector Count Mode 3 (DWord)
    // unsigned short Write_Read_Verify_Sector_Count_Mode_3[2];
    // //  212-213 Write-Read-Verify Sector Count Mode 2 (DWord)
    // unsigned short Write_Read_Verify_Sector_Count_Mode_2[2];
    // //  214 NV Cache Capabilities
    // unsigned short NV_Cache_Capabilities;
    // //  215-216 NV Cache Size in Logical Blocks (DWord)
    // unsigned short NV_Cache_Size[2];
    // //  217 Nominal media rotation rate
    // unsigned short Nominal_media_rotation_rate;
    // //  218 Reserved
    // unsigned short Reserved11;
    // //  219 NV Cache Options
    // unsigned short NV_Cache_Options;
    // //  220 Write-Read-Verify feature set current mode
    // unsigned short Write_Read_Verify_feature_set_current_mode;
    // //  221 Reserved
    // unsigned short Reserved12;
    // //  222 Transport major version number.
    // //      0000h or ffffh = device does not report version
    // unsigned short Transport_Major_Version_Number;
    // //  223 Transport Minor version number
    // unsigned short Transport_Minor_Version_Number;
    // //  224-233 Reserved for CE-ATA
    // unsigned short Reserved13[10];
    // //  234 Minimum number of 512-byte data blocks per DOWNLOAD MICROCODE command for mode 03h
    // unsigned short Mini_blocks_per_CMD;
    // //  235 Maximum number of 512-byte data blocks per DOWNLOAD MICROCODE command for mode 03h
    // unsigned short Max_blocks_per_CMD;
    // //  236-254 Reserved
    // unsigned short Reserved14[19];
    // //  255 Integrity word
    // //      15:8    Checksum
    // //      7:0 Checksum Validity Indicator
    // unsigned short Integrity_word;
}