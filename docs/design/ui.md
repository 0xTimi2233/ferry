---
name: 模型调用网关设计系统
description: 面向开发者的深色界面，以数据密度和状态可见性为第一优先
colors:
  primary: "#22D3EE"
  secondary: "#94A3B8"
  tertiary: "#818CF8"
  neutral: "#64748B"
  background-100: "#0A0F1A"
  background-200: "#0C1320"
  surface: "#111A2B"
  on-surface: "#E6ECF5"
  outline: "#1D2A42"
  error: "#F87171"
typography:
  display-lg:
    fontFamily: Inter, system-ui, sans-serif
    fontSize: 20px
    fontWeight: "600"
    lineHeight: 28px
  body-md:
    fontFamily: Inter, system-ui, sans-serif
    fontSize: 14px
    lineHeight: 21px
  label-sm:
    fontFamily: Inter, system-ui, sans-serif
    fontSize: 11px
    lineHeight: 17px
rounded:
  sm: 8px
  md: 10px
  lg: 14px
  full: 9999px
spacing:
  sm: 8px
  md: 16px
  lg: 20px
  xl: 32px
---

# 视觉设计系统

## 设计理念与基调

深色为主，界面本身退到背景，数据与状态站到前面。层级靠表面亮度递增加 1px 细边框表达，不用阴影。等宽字体只用于标识符、模型名与密钥，数字列启用等宽数字并右对齐。

令牌沿四条正交轴线组织，改动其一不牵连其余：颜色、圆角、密度、字体。颜色由主色派生强调色与状态色，不写死十六进制。

内容区不设最大宽度，宽屏下网格与图表拉伸填满可用空间，靠间距与对齐线组织，不靠容器堆叠。

## 组件形态基准

- 按钮：主操作用 primary 实心，次级操作用 outline 描边，危险操作用 error 描边且仅在与删除相关的场景出现
- 容器：卡片用 surface 底色加 outline 细边框，圆角 lg，内边距 lg，不使用投影
- 状态标记：健康用圆点加文字，冷却用 primary 描边胶囊，失效用 error 描边胶囊
- 表格：默认分隔线加悬停高亮，数值右对齐，行内操作不超过两个，其余收进菜单
- 指标卡：标签在上、数值在下并底部对齐，次级说明与趋势线同排

## 密度档位

行高按断点收放，同一张表内不混用：

| 档位 | 行高 | 内边距 |
|---|---|---|
| 紧凑 | 32px | 16px |
| 默认 | 40px | 20px |
| 宽松 | 44px | 20px |
