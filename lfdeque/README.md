### Ideas
- For the expandable q, try making the seat a `Arc<Refcell<T>>`. Then on the get, get a clone to each arc which should get you a ref to the underlying value. Finally we can run https://doc.rust-lang.org/std/cell/struct.RefCell.html#method.take on the thing that wins the contest. 
