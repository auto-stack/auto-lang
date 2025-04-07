enum Session {
    Default = 0x01
    Programming = 0x02
    Extended = 0x03
}

enum SecurityLevel {
    Locked = 0x00
    Level1 = 0x01
    Level2 = 0x02
    All = 0xFF
}

enum Service {
    SessionControl = 0x10,
    EcuReset = 0x11,
    ClearDTC = 0x14,
    ReadDTC = 0x19,
    ReadData = 0x22,
    ReadMemory = 0x23,
    SecurityAccess = 0x27,
    ComControl = 0x28,
    ReadPeriodic = 0x2A,
    DynamicDefine = 0x2C,
    WriteData = 0x2E,
    IOControl = 0x2F,
    RoutineControl = 0x31,
    WriteMemory = 0x3D,
    TesterPresent = 0x3E,
    ControlDTC = 0x85,
}

var ServiceInfo = [
    { id: 0x10, name: "DiagnosticSessionControl",  desc: "诊断会话控制" },
    { id: 0x11, name: "EcuReset",  desc: "电控单元复位" },
    { id: 0x14, name: "ClearDiagnosticInformation",  desc: "清除诊断信息" },
    { id: 0x19, name: "ReadDTCInformation",  desc: "读取DTC信息" },
    { id: 0x22, name: "ReadDataByIdentifier",  desc: "读取数据" },
    { id: 0x23, name: "ReadMemoryByAddress",  desc: "读取内存" },
    { id: 0x27, name: "SecurityAccess",  desc: "安全访问" },
    { id: 0x28, name: "CommunicationControl",  desc: "通信控制 " },
    { id: 0x2A, name: "ReadDataByPeriodicIdentifier",  desc: "读取数据（周期标识符）" },
    { id: 0x2C, name: "DynamicallyDefineDataIdentifier",  desc: "动态定义数据标识符" },
    { id: 0x2E, name: "WriteDataByIdentifier",  desc: "写入数据" },
    { id: 0x2F, name: "InputOutputControlByIdentifier",  desc: "输入输出控制" },
    { id: 0x31, name: "RoutineControl",  desc: "例程控制" },
    { id: 0x3D, name: "WriteMemoryByAddress",  desc: "写入内存" },
    { id: 0x3E, name: "TesterPresent",  desc: "诊断设备在线" },
    { id: 0x85, name: "ControlDTCSetting",  desc: "控制DTC设置" },
]

enum RoutineAction {
    Start = 0x01,
    Stop = 0x02,
    Result = 0x03,
}

var routines = [
    {
        id: 0x6304
        action: [Start, Stop, Result]
        class: "Routine_Control"
        has_stop: true
        name: "ResetVehicleMaintenance"
        start_arg: 0
        start_ret: 1
        stop_arg: 0
        stop_ret: 1
        result_arg: 0
        result_ret: 1
    }
    {
        id: 0x6305
        action: [Start, Stop, Result]
        class: "Routine_Control"
        has_stop: true
        name: "SetTheFirstMileage"
        start_arg: 1
        start_ret: 1
        stop_arg: 1
        stop_ret: 1
        result_arg: 1
        result_ret: 1
    }
    {
        id: 0x6306
        action: [Start, Stop, Result]
        class: "Routine_Control"
        has_stop: true
        name: "SetMaintenanceInterval"
        start_arg: 1
        start_ret: 1
        stop_arg: 1
        stop_ret: 1
        result_arg: 1
        result_ret: 1
    }
]

