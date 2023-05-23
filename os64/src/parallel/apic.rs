

pub struct pt_regs {
	r15 : u64,
	r14 : u64,
	r13 : u64,
	r12 : u64,
	r11 : u64,
	r10 : u64,
	r9 : u64,
	r8 : u64,
	rbx : u64,
	rcx : u64,
	rdx : u64,
	rsi : u64,
	rdi : u64,
	rbp : u64,
	ds : u64,
	es : u64,
	rax : u64,
	func : u64,
	errcode : u64,
	rip : u64,
	cs : u64,
	rflags : u64,
	rsp : u64,
	ss : u64,
}

use bitfield::bitfield;

bitfield!{
    pub struct IO_APIC_RET_entry(MSB0 [u8]);
    u32;
    get_vector, set_vector: 7, 0;
    get_deliver_mode, set_deliver_mode: 10, 8;
    get_dest_mode, set_dest_mode: 11, 11;
    get_deliver_status, set_deliver_status : 12, 12;    
    get_polarity,set_polarity: 13, 13;
    get_irr,set_irr: 14, 14;
    get_trigger,set_trigger: 15, 15;
    get_mask,set_mask: 16, 16;
    get_reserved,set_reserved: 31, 17;
    get_reserved1,set_reserved1: 55, 32;
    get_dest,set_dest:63,56;
}

pub const APIC_ICR_IOAPIC_FIXED : u8 = 0;	//LAPIC	IOAPIC 	ICR
pub const IOAPIC_ICR_LOWEST_PRIORITY : u8 = 1;	//	IOAPIC 	ICR
pub const APIC_ICR_IOAPIC_SMI : u8 = 2;	//LAPIC	IOAPIC 	ICR

pub const APIC_ICR_IOAPIC_NMI : u8 = 4;	//LAPIC	IOAPIC 	ICR
pub const APIC_ICR_IOAPIC_INIT : u8 =  5;	//LAPIC	IOAPIC 	ICR
pub const ICR_START_UP : u8 = 6;	//		ICR
pub const IOAPIC_EXT_INT : u8 = 7;	//	IOAPIC

//timer mode
pub const APIC_LVT_TIMER_ONE_SHOT : u8 = 0;
pub const APIC_LVT_TIMER_PERIODIC : u8 = 1;
pub const APIC_LVT_TIMER_TSC_DEADLINE : u8 = 2;

//mask
pub const APIC_ICR_IOAPIC_MASKED : u8 = 1;
pub const APIC_ICR_IOAPIC_UN_MASKED : u8 = 0;

//trigger mode
pub const APIC_ICR_IOAPIC_EDGE : u8 = 0;
pub const APIC_ICR_IOAPIC_LEVEL : u8 = 1;

//delivery status
pub const APIC_ICR_IOAPIC_IDLE : u8 = 0;
pub const APIC_ICR_IOAPIC_SEND_PENDING : u8 = 1;

//destination shorthand
pub const ICR_NO_SHORTHAND : u8 = 0;
pub const ICR_SELF : u8 = 1;
pub const ICR_ALL_INCLUDE_SELF : u8 = 2;
pub const ICR_ALL_EXCLUDE_SELF : u8 = 3;

//destination mode
pub const ICR_IOAPIC_DELV_PHYSICAL : u8 = 0;
pub const ICR_IOAPIC_DELV_LOGIC : u8 = 1;

//level
pub const ICR_LEVEL_DE_ASSERT : u8 = 0;
pub const ICR_LEVLE_ASSERT : u8 = 1;

//remote irr
pub const APIC_IOAPIC_IRR_RESET : u8 = 0;
pub const APIC_IOAPIC_IRR_ACCEPT : u8 = 1;

//pin polarity
pub const APIC_IOAPIC_POLARITY_HIGH : u8 = 0;
pub const APIC_IOAPIC_POLARITY_LOW : u8 = 1;

/*

0~255 IDT

0   ~   31	trap fault abort for system
	0	devide error
	1	debug
	2	NMI
	3	breakpoint
	4	overflow
	5	bound range
	6	undefined opcode
	7	device	not available
	8	double fault
	9	coprocessor segment overrun
	10	invalid TSS
	11	segment not present
	12	stack segment fault
	13	general protection
	14	page fault
	15	
	16	x87 FPU error
	17	alignment check
	18	machine check
	19	SIMD exception
	20	virtualization exception
21  ~   31	Do not use

32  ~   55	I/O APIC
	32	8259A
	33	keyboard
	34	HPET timer 0,8254 counter 0
	35	serial port A
	36	serial port B
	37	parallel port
	38	floppy
	39	parallel port
	40	RTC,HPET timer 1
	41	Generic
	42	Generic
	43	HPET timer 2
	44	HPET timer 3
	45	FERR#
	46	SATA primary
	47	SATA secondary
	48	PIRQA
	49	PIRQB
	50	PIRQC
	51	PIRQD
	52	PIRQE
	53	PIRQF
	54	PIRQG
	55	PIRQH
	
	
0x80		system call

150 ~   200	Local APIC
	150	CMCI
	151	Timer
	152	Thermal Monitor
	153	Performance Counter
	154	LINT0
	155	LINT1
	156	Error

200 ~   255	MP IPI

*/

/*

1:	LVT	CMCI
2:	LVT	Timer
3:	LVT	Thermal Monitor
4:	LVT	Performace Counter
5:	LVT	LINT0
6:	LVT	LINT1
7:	LVT	Error

*/

/*
	LVT
*/

// struct APIC_LVT
// {
// 	unsigned int 	vector	:8,	//0~7	ALL
// 		deliver_mode	:3,	//8~10	      CMCI LINT0 LINT1 PerformCounter ThermalSensor
// 			res_1	:1,	//11
// 		deliver_status	:1,	//12	ALL
// 			polarity:1,	//13	           LINT0 LINT1
// 			irr	:1,	//14	           LINT0 LINT1
// 			trigger	:1,	//15	           LINT0 LINT1
// 			mask	:1,	//16	ALL
// 		timer_mode	:2,	//17~18	Timer
// 			res_2	:13;	//19~31
// }__attribute__((packed));		//disable align in struct

/*
	ICR
*/

// struct INT_CMD_REG
// {
// 	unsigned int 	vector	:8,	//0~7
// 		deliver_mode	:3,	//8~10
// 		dest_mode	:1,	//11
// 		deliver_status	:1,	//12
// 			res_1	:1,	//13
// 			level	:1,	//14
// 			trigger	:1,	//15
// 			res_2	:2,	//16~17
// 		dest_shorthand	:2,	//18~19
// 			res_3	:12;	//20~31
	
// 	union {
// 		struct {
// 			unsigned int	res_4	:24,	//32~55
// 			dest_field	:8;	//56~63		
// 			}apic_destination;
			
// 		unsigned int x2apic_destination;	//32~63
// 		}destination;
		
// }__attribute__((packed));



// //delivery mode
// #define	APIC_ICR_IOAPIC_Fixed 		 0	//LAPIC	IOAPIC 	ICR
// #define	IOAPIC_ICR_Lowest_Priority 	 1	//	IOAPIC 	ICR
// #define	APIC_ICR_IOAPIC_SMI		 2	//LAPIC	IOAPIC 	ICR

// #define	APIC_ICR_IOAPIC_NMI		 4	//LAPIC	IOAPIC 	ICR
// #define	APIC_ICR_IOAPIC_INIT		 5	//LAPIC	IOAPIC 	ICR
// #define	ICR_Start_up			 6	//		ICR
// #define	IOAPIC_ExtINT			 7	//	IOAPIC


// //timer mode
// #define APIC_LVT_Timer_One_Shot		0
// #define APIC_LVT_Timer_Periodic		1
// #define APIC_LVT_Timer_TSC_Deadline	2

// //mask
// #define APIC_ICR_IOAPIC_Masked		1
// #define APIC_ICR_IOAPIC_UN_Masked	0

// //trigger mode
// #define APIC_ICR_IOAPIC_Edge		0
// #define APIC_ICR_IOAPIC_Level		1

// //delivery status
// #define APIC_ICR_IOAPIC_Idle		0
// #define APIC_ICR_IOAPIC_Send_Pending	1

// //destination shorthand
// #define ICR_No_Shorthand		0
// #define ICR_Self			1
// #define ICR_ALL_INCLUDE_Self		2
// #define ICR_ALL_EXCLUDE_Self		3

// //destination mode
// #define ICR_IOAPIC_DELV_PHYSICAL	0
// #define ICR_IOAPIC_DELV_LOGIC		1

// //level
// #define ICR_LEVEL_DE_ASSERT		0
// #define ICR_LEVLE_ASSERT		1

// //remote irr
// #define APIC_IOAPIC_IRR_RESET		0
// #define APIC_IOAPIC_IRR_ACCEPT		1

// //pin polarity
// #define APIC_IOAPIC_POLARITY_HIGH	0
// #define APIC_IOAPIC_POLARITY_LOW	1

/* 

*/

// struct IOAPIC_map
// {
// 	unsigned int physical_address;
// 	unsigned char * virtual_index_address;
// 	unsigned int *  virtual_data_address;
// 	unsigned int *  virtual_EOI_address;
// }ioapic_map;

// unsigned long ioapic_rte_read(unsigned char index);
// void ioapic_rte_write(unsigned char index,unsigned long value);

/* 

*/

// void IOAPIC_pagetable_remap();

// /*

// */

// void do_IRQ(struct pt_regs * regs,unsigned long nr);

// /*

// */

// void APIC_IOAPIC_init();
// void Local_APIC_init();
// void IOAPIC_init();

// void IOAPIC_enable(unsigned long irq);
// void IOAPIC_disable(unsigned long irq);
// unsigned long IOAPIC_install(unsigned long irq,void * arg);
// void IOAPIC_uninstall(unsigned long irq);
// void IOAPIC_level_ack(unsigned long irq);
// void IOAPIC_edge_ack(unsigned long irq);
