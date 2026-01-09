use frand_property::slint_model;
use crate::{AdderVecGlobal, AdderVecGlobalData};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use futures::future::BoxFuture;

const PROP_LEN: usize = 3;

slint_model! {
    pub AdderVecModel: AdderVecGlobal {
        in values: i32[PROP_LEN],
        out sum: i32,
    }
}

impl<C: slint::ComponentHandle + 'static> AdderVecModel<C> {
    pub fn start(&self) {
        let values = self.values.clone();
        let sum = self.sum.clone();

        crate::spawn(async move {
            let mut futures: FuturesUnordered<BoxFuture<'static, (usize, i32)>> = FuturesUnordered::new();
            
            // 초기 퓨처 등록
            for (idx, rx) in values.iter().enumerate() {
                let mut rx = rx.clone();
                futures.push(Box::pin(async move {
                    let val = rx.changed().await;
                    (idx, val)
                }));
            }

            // 변경 감지 루프
            while let Some((idx, _val)) = futures.next().await {
                // 변경된 인덱스의 퓨처 재등록
                let mut rx = values[idx].clone();
                futures.push(Box::pin(async move {
                    let val = rx.changed().await;
                    (idx, val)
                }));

                // 합계 재계산 및 전송
                let current_sum: i32 = values.iter().map(|v| v.value()).sum();
                sum.send(current_sum);
            }
        });
    }
}
