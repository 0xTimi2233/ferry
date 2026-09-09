---
name: 模型调用网关设计系统
description: 面向开发者的深色界面，以数据密度和状态可见性为第一优先
colors:
  primary: "#22D3EE"
  secondary: "#94A3B8"
  tertiary: "#818CF8"
  neutral: "#64748B"
  background-100: "#0B1120"
  background-200: "#111C2E"
  surface: "#16233A"
  on-surface: "#E2E8F0"
  outline: "#24334D"
  error: "#F87171"
typography:
  display-lg:
    fontFamily: Inter, system-ui, sans-serif
    fontSize: 28px
    fontWeight: "600"
    lineHeight: 36px
  body-md:
    fontFamily: Inter, system-ui, sans-serif
    fontSize: 14px
    lineHeight: 22px
  label-sm:
    fontFamily: Inter, system-ui, sans-serif
    fontSize: 12px
    lineHeight: 18px
rounded:
  sm: 6px
  md: 10px
  full: 9999px
spacing:
  sm: 8px
  md: 16px
  lg: 24px
---

# 视觉设计系统

## 设计理念与基调

深色为主，界面本身退到背景，数据与状态站到前面。层级靠背景色深浅区分，不靠边框和阴影堆叠。等宽字体只用于标识符、模型名与密钥，正文保持无衬线。

## 组件形态基准

- 按钮：主操作用 primary 实心，次级操作用 outline 描边，危险操作用 error 描边且仅在与删除相关的场景出现
- 容器：卡片用 surface 底色加 outline 细边框，圆角 md，不使用投影
- 状态标记：健康用圆点加文字，冷却用 primary 描边胶囊，失效用 error 描边胶囊
