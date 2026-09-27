# PRD 需求诊断说明书: tauri-apps/tauri #2048

## 1. 核心缺陷机理与根因剖析
Master/Slave PTY 双向管道在 EOF 或退出时读端未正确释放句柄，导致工作线程阻塞。

## 2. 影响模块与范围
- tauri-apps/tauri/core
- tauri-apps/tauri/api

## 3. PRD 技术实现方案与修改蓝图
引入防御性前置校验与精准异常拦截，避免底层错误击穿服务层。

## 4. TDD 自动化测试验证策略
编写端到端单元测试与异常模拟用例，验证各种边界值与中断行为。

## 5. 需求与代码关系对照矩阵
- 需求点: 解决 Issue #2048: Refactor PTY master slave duplex pipes on Unix platforms ➔ 模块: ["tauri-apps/tauri/core", "tauri-apps/tauri/api"] ➔ 缺陷: Master/Slave PTY 双向管道在 EOF 或退出时读端未正确释放句柄，导致工作线程阻塞。 ➔ 改动: 引入防御性前置校验与精准异常拦截，避免底层错误击穿服务层。

## 6. 具体实装任务清单
- [ ] 检查并修正受影响模块: ["tauri-apps/tauri/core", "tauri-apps/tauri/api"]
- [ ] 实装核心防御性修复: 引入防御性前置校验与精准异常拦截，避免底层错误击穿服务层。
- [ ] 按测试策略「编写端到端单元测试与异常模拟用例，验证各种边界值与中断行为。」增加回归单测
