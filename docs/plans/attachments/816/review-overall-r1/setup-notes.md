# 运行边界与排除项

- 当前真实工具的构建错误是proc_host.rs的E0063，发生在下述文件锁实验前；不是锁错误。独立CLI构建先成功，其hash与service/json/positive-driver采用的副本绑定。
- 后续发现root auto.exe被既有测试daemon PID7240占用（pipe `autovm-p816-driver-22508`；原父PID已不存在）。未结束该实例，不把共享exe文件锁记为产品finding。原路径Cargo driver构建因此报os error5，改用相同源码test文件、相同hash CLI副本的独立test-only manifest；该正控边界写在positive-driver结果中。
- 初版异常watch包把bin命名auto并误用了根target，遭上述锁拒绝，未成功覆盖真实CLI。最终异常包改为私有isolated-target；旧失败日志后由有效最终日志替换，本文保留其来源。正常源码/机器JSON脚本的原始CLIhash仍一致。
- 初版WSL env调用找不到cargo（非login PATH）；修正为已验证的/home/visus/.cargo/bin路径后原生Linux成功执行，实际InvalidSpec结果才用于F03。未把PATH错误作为产品问题。
- JSON info样例没有pac.atom，exit1属于合法错误输入；仅将new命令exit0的混流用作成功JSON反例。
- service-cargo宿主仅用于owned退出异常，故意无HTTP监听；不把它的READY行当真实HTTP正控，也不替代work尚未完成的真实Rust服务验收。
- watch异常宿主实际保留旧产物、只发进度文字；它只检验原测试的区分力。正控使用本轮真实CLI同字节副本，未把异常宿主当正常CLI证据。
