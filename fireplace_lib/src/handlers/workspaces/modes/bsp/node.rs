trait NodeAdditions {
    fn is_child(&self, node: &NodeId, direction: Option<Direction>) -> bool;
    fn direction_of_child(&self, node: &NodeId) -> Option<Direction>;
    fn other_child(&self, node: &NodeId) -> Option<&NodeId>;
    fn lowest_view(&self, tree: &Tree<Data>, direction: Direction) -> WeakView;
    fn number_of_children(&self, tree: &Tree<Data>) -> usize;
    fn orientation(&self) -> Option<Orientation>;
    // fn print_tree(&self, tree: &Tree<Data>, format: &mut String);
}

impl NodeAdditions for Node<Data> {
    fn is_child(&self, node: &NodeId, direction: Option<Direction>) -> bool {
        match direction {
            Some(Direction::Left) => self.children().get(0) == Some(node),
            Some(Direction::Right) => self.children().get(1) == Some(node),
            None => self.children().contains(node),
        }
    }

    fn direction_of_child(&self, node: &NodeId) -> Option<Direction> {
        if self.is_child(node, Some(Direction::Left)) {
            Some(Direction::Left)
        } else if self.is_child(node, Some(Direction::Right)) {
            Some(Direction::Right)
        } else {
            None
        }
    }

    fn other_child(&self, node: &NodeId) -> Option<&NodeId> {
        if self.children().get(0) == Some(node) {
            self.children().get(1)
        } else if self.children().get(1) == Some(node) {
            self.children().get(0)
        } else {
            None
        }
    }

    fn lowest_view(&self, tree: &Tree<Data>, direction: Direction) -> WeakView {
        match *self.data() {
            Data::Split(_) => {
                let child = self.children().get(if direction == Direction::Left { 0 } else { 1 }).unwrap();

                tree.get(child).unwrap().lowest_view(tree, direction)
            }
            Data::Leaf(Leaf { ref view }) => view.clone(),
        }
    }

    fn number_of_children(&self, tree: &Tree<Data>) -> usize {
        match *self.data() {
            Data::Split(_) => {
                self.children().iter().fold(0,
                                            |i, child| i + tree.get(child).unwrap().number_of_children(tree))
            }
            Data::Leaf(_) => 1,
        }
    }

    fn orientation(&self) -> Option<Orientation> {
        match *self.data() {
            Data::Split(Split { ref orientation, .. }) => Some(*orientation),
            _ => None,
        }
    }

    // fn print_tree(&self, tree: &Tree<Data>, format: &mut String) {
    // match self.data() {
    // x @ &Data::Split(_) => {
    // format.push_str(&format!("[ ({:?})\n", x));
    // for child in self.children()
    // {
    // tree.get(&child).unwrap().print_tree(tree, format);
    // format.push_str(",\n");
    // }
    // format.push_str("]");
    // },
    // x => format.push_str(&format!("{:?}", x)),
    // }
    // }
    //
}
