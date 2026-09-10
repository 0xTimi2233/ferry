//! 领域事件发布端口
//!
//! 切片之间不存在代码级依赖。跨模块协作只有一条通道：发布方把领域事件交给本端口，
//! 组装根在装配时把订阅方注册到发布端口的实现上，订阅方以用例的形式消费事件。
//! 端口只提供发布方向，订阅注册属于装配，不属于任何切片。

use async_trait::async_trait;

use super::PortError;
use crate::domain::events::DomainEvent;

/// 发布领域事件，实现由组装根注入
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &DomainEvent) -> Result<(), PortError>;
}
