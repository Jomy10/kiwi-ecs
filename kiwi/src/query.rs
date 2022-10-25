use crate::ArchetypeId;

pub struct QueryResult {
    archetypes: Vec<ArchetypeId>, // Created because of borrowing rules (sadly, but no way to fix it that I can see)
}

// use crate::Component;
// use std::iter::Iterator;
// use std::marker::PhantomData;

// pub trait IteratorTuple<CT: ComponentTuple> {
//     fn next(&mut self) -> Option<CT>;
// }
// pub trait ComponentTuple {}

// pub struct ComponentsIterator<
//     // 'a,
//     CompTuple: ComponentTuple,
//     IterTuple: IteratorTuple<CompTuple>,
//     Iter: Iterator<Item = IterTuple>
// > {
//     iterator: Iter,
//     current_iterator: Option<IterTuple>,
//     return_type: PhantomData<CompTuple>
// }

// impl<
//     // 'a,
//     CompTuple: ComponentTuple,
//     IterTuple: IteratorTuple<CompTuple>,
//     Iter: Iterator<Item = IterTuple>
// > Iterator for ComponentsIterator<CompTuple, IterTuple, Iter> {
//     type Item = CompTuple;

//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             if let Some(tuple) = &mut self.current_iterator {
//                 let next = tuple.next();
//                 if next.is_none() {
//                     self.current_iterator = self.iterator.next();
//                 } else {
//                     return next;
//                 }
//             } else {
//                 self.current_iterator = self.iterator.next();
//                 if self.current_iterator.is_none() {
//                     return None;
//                 }
//             }
//         }
//    }
// }

// //===============
// // Tuples
// //===============
// pub struct ComponentTuple1<'a, A: Component>(&'a A);
// impl<'a, A: Component> ComponentTuple for ComponentTuple1<'a, A> {}
// pub struct ComponentTuple2<'a, A: Component, B: Component>(&'a A, &'a B);
// impl<'a, A: Component, B: Component> ComponentTuple for ComponentTuple2<'a, A, B> {}

// //===============
// // Iter tuples
// //===============

// pub struct IteratorTuple1<'a, A: Component + 'static, IA: Iterator<Item = &'a A>> {
//     iterator_a: IA
// }
// impl<'a,
//     A: Component +'static,
//     IA: Iterator<Item = &'a A>,
// > IteratorTuple<ComponentTuple1<'a, A>> for IteratorTuple1<'a, A, IA> {
//     fn next(&mut self) -> Option<ComponentTuple1<'a, A>> {
//         let a = self.iterator_a.next();
//         if a.is_none() {
//             return None;
//         }
//         Some(ComponentTuple1(unsafe { a.unwrap_unchecked() }))
//     }
// }
// pub struct IteratorTuple2<
//     'a,
//     A: Component +'static,
//     IA: Iterator<Item = &'a A>,
//     B: Component +'static,
//     IB: Iterator<Item = &'a B>
// > {
//     iterator_a: IA,
//     iterator_b: IB
// }
// impl<'a,
//     A: Component +'static,
//     IA: Iterator<Item = &'a A>,
//     B: Component +'static,
//     IB: Iterator<Item = &'a B>,
// > IteratorTuple<ComponentTuple2<'a, A, B>> for IteratorTuple2<'a, A, IA, B, IB> {
//     fn next(&mut self) -> Option<ComponentTuple2<'a, A, B>> {
//         let a = self.iterator_a.next();
//         if a.is_none() {
//             return None;
//         }
//         unsafe {Some(ComponentTuple2(
//             a.unwrap_unchecked(),
//             self.iterator_b.next().unwrap_unchecked()
//         ))}
//     }
// }

// /*
// use std::sync::RwLockReadGuard;
// use std::iter::Iterator;
// use std::marker::PhantomData;
// use std::mem::MaybeUninit;
// use crate::{Component, ArchetypeId, EntityId};

// pub struct ComponentsIterator<'a, Q: QueryYield> {
//     components_guard: RwLockReadGuard<'a, Vec<ArchetypeId>>,
//     arch_id_iterator: impl Iterator<Item = u32>,
//     return_type: PhantomData<Q>,
// }

// impl<'a, Q: QueryYield> ComponentsIterator<'a, Q> {
//     pub fn new(guards: Vec<&RwLockReadGuard<'a, Vec<ArchetypeId>>>) -> Self {
        
//         Self {
//             components_guard: guard,
//             arch_id_iterator: guard.iter().filter(|elem|)
//         }
//     }
// }

// impl<'a, Q: QueryYield> Iterator for ComponentsIterator<'a, Q> {
//     type Item = Q;

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }

// trait QueryYield {}


// struct QueryYield1<'a, A: Component>(&'a A);
// struct QueryYield2<'a, A: Component, B: Component>(&'a A, &'a B);
// struct QueryYield1Id<'a, A: Component>(EntityId, &'a A);
// */
// /*

// pub struct ComponentsIterator
// <
//     'a,
//     A: Component + 'static,
//     B: Component + 'static,
//     IA: Iterator<Item = &'a A>,
//     IB: Iterator<Item = &'a B>,
//     I: Iterator<Item = (IA, IB)>
// > {
//     val: I,
//     current: Option<(IA, IB)>
// }

// impl<
//     'a, 
//     A: Component + 'static,
//     B: Component + 'static, 
//     IA: Iterator<Item = &'a A>,
//     IB: Iterator<Item = &'a B>,
//     I: Iterator<Item = (IA, IB)>,
// > ComponentsIterator<'a, A, B, IA, IB, I> {
//     // TODO: pub(crate)
//     pub fn new(val: I) -> Self {
//         Self {
//             val,
//             current: None,
//         }
//     }
// }

// impl<
//     'a, 
//     A: Component + 'static,
//     B: Component + 'static, 
//     IA: Iterator<Item = &'a A>,
//     IB: Iterator<Item = &'a B>,
//     I: Iterator<Item = (IA, IB)>,
// > Iterator for ComponentsIterator<'a, A, B, IA, IB, I> {
//     type Item = (&'a A, &'a B);

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.current.is_none() {
//             let new_cur = self.val.next();
//             self.current = Some(unsafe { new_cur.unwrap_unchecked() });
//         }
        
//         let mut current = unsafe { self.current.as_mut().unwrap_unchecked() };
//         let mut a = current.0.next();
        
//         while a.is_none() {
//             let new_cur = self.val.next();
//             if new_cur.is_none() {
//                 return None;
//             }
//             self.current = Some(unsafe { new_cur.unwrap_unchecked() });
//             current = unsafe { self.current.as_mut().unwrap_unchecked() };
//             a = current.0.next();
//         }
//         let b = current.1.next();
//         return Some(unsafe { (a.unwrap_unchecked(), b.unwrap_unchecked()) });
//     }
// }
// */
