
use protocols::example::shoppingcart::{
    AddLineItem, RemoveLineItem, GetShoppingCart, Cart,
    persistence::{self, ItemAdded, ItemRemoved, LineItem}
};

fn main() -> () {

}

pub struct ShoppingCartEntity(Cart);

trait Context {
    fn emit<T>(event: T);
    fn fail(message: String);
}

//TODO Spike a possible alternative entity interface implementation
impl ShoppingCartEntity {
    // it need to be checked with a macro whether all the command or event handlers are implemented
    fn add_line_item(&self, cmd: AddLineItem, cx: &mut impl Context) {}
    fn remove_line_item(&self, cmd: RemoveLineItem, cx: &mut impl Context) {}
    fn get_shopping_cart(&self, cmd: GetShoppingCart, cx: &mut impl Context) -> Cart {
        unimplemented!();
    }

    fn item_added(&mut self, evt: ItemAdded) {}
    fn item_removed(&mut self, evt: ItemRemoved) {}

    fn restore(&mut self, cart: persistence::Cart) {}
    fn snapshot(&self) -> persistence::Cart {
        let items = self.0.items.iter().map(|item|
            persistence::LineItem {
                product_id: item.product_id.clone(),
                name: item.name.clone(),
                quantity: item.quantity,
            }
        ).collect();
        persistence::Cart {
            items,
        }
    }
}
