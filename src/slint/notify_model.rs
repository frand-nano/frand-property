use std::rc::Rc;
use slint::{Model, ModelNotify, ModelTracker, VecModel};

pub struct SlintNotifyModel<T> {
    inner: Rc<VecModel<T>>,
    notify: ModelNotify,
    on_change: Box<dyn Fn(usize, T)>,
}

impl<T> SlintNotifyModel<T> {
    pub fn new(inner: Rc<VecModel<T>>, on_change: impl Fn(usize, T) + 'static) -> Self {
        Self {
            inner,
            notify: ModelNotify::default(),
            on_change: Box::new(on_change),
        }
    }
}

impl<T: Clone + 'static + PartialEq> Model for SlintNotifyModel<T> {
    type Data = T;

    fn row_count(&self) -> usize {
        self.inner.row_count()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.inner.row_data(row)
    }

    fn set_row_data(&self, row: usize, data: Self::Data) {
        // 실제로 데이터가 변경되었는지 확인하여 무한 루프 방지
        if let Some(current) = self.inner.row_data(row) {
            if current == data {
                return;
            }
        }

        self.inner.set_row_data(row, data.clone());
        self.notify.row_changed(row);
        (self.on_change)(row, data);
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
