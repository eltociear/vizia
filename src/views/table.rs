use std::rc::Rc;

use morphorm::LayoutType;

use crate::{Context, HStack, Handle, ItemPtr, Lens, Model, Store, TreeExt, View};
use crate::Units::*;

pub struct Table<L,T: 'static> 
where L: Lens<Target = Vec<T>>,
{

    width: usize,
    lens: L,
    builder: Option<Rc<dyn Fn(&mut Context, usize, ItemPtr<L,T>)>>

}

impl<L: 'static + Lens<Target = Vec<T>>,T> Table<L,T> {
    pub fn new<F>(cx: &mut Context, width: usize, lens: L, builder: F) -> Handle<Self> 
    where
        F: 'static + Fn(&mut Context, usize, ItemPtr<L, T>),
        <L as Lens>::Source: Model,
    {
        Self {
            lens,
            width,
            builder: Some(Rc::new(builder)),
        }.build(cx).layout_type(LayoutType::Grid).row_between(Pixels(1.0)).col_between(Pixels(1.0))
    }
}

impl<L, T> View for Table<L, T> 
where L: 'static + Lens<Target = Vec<T>>
{
    fn body(&mut self, cx: &mut Context) {
        for child in cx.current.child_iter(&cx.tree.clone()) {
            cx.remove(child);
        }

        let builder = self.builder.take().unwrap();

        let mut found_store = None;

        'tree: for entity in cx.current.parent_iter(&cx.tree.clone()) {
            if let Some(model_list) = cx.data.model_data.get(entity) {
                for (_, model) in model_list.iter() {
                    if let Some(store) = model.downcast_ref::<Store<L::Source>>() {
                        found_store = Some(store); 
                        break 'tree;
                    }
                }
            }
        };

        if let Some(store) = found_store {
            
            let len = self.lens.view(&store.data).len();

            assert!(len/self.width == self.width, "Only square tables supported at the moment");

            cx.style.borrow_mut().grid_rows.insert(cx.current, vec![Stretch(1.0); self.width]);
            cx.style.borrow_mut().grid_cols.insert(cx.current, vec![Stretch(1.0); self.width]);

            for row in 0..len/self.width {
                for col in 0..self.width {
                    let ptr = ItemPtr::new(self.lens.clone(), row*self.width + col, row, col);
                    let width = self.width;
                    let builder = builder.clone();
                    HStack::new(cx, move |cx|{
                        (builder)(cx, width, ptr.clone());
                    }).row_index(row).col_index(col);
                    cx.count += 1;
                }
            }
        }

    }
}