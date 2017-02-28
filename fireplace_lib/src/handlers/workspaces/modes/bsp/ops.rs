impl BSP {
    fn recalculate(&mut self) {
        debug!(slog_scope::logger(), "Recalculating Tree");

        let root_id = self.tree.root_node_id().cloned();
        if let Some(root_id) = root_id {
            let geo = self.size;
            self.calculate_node(&root_id, geo);
        }
    }

    fn calculate_node(&mut self, id: &NodeId, remaining: Geometry) {
        let node: Data = self.tree.get(id).unwrap().data().clone();

        match node {
            Data::Leaf(Leaf { ref view }) => {
                view.run(|view| {
                    let lock = view.get::<UsableViewGeometry>();
                    view.set_geometry(ResizeEdge::Flags::empty(),
                                      match lock.as_ref().and_then(|x| x.read().ok()) {
                                          Some(scissor) => {
                                              Geometry {
                                                  origin: Point {
                                                      x: remaining.origin.x / view.output().scale() as i32 +
                                                         (scissor.left as u32) as i32,
                                                      y: remaining.origin.y / view.output().scale() as i32 +
                                                         (scissor.up as u32) as i32,
                                                  },
                                                  size: Size {
                                                      w: remaining.size.w / view.output().scale() -
                                                         (scissor.left as u32) -
                                                         (scissor.right as u32),
                                                      h: remaining.size.h / view.output().scale() -
                                                         (scissor.up as u32) -
                                                         (scissor.down as u32),
                                                  },
                                              }
                                          }
                                          None => remaining,
                                      });
                });
            }

            Data::Split(Split { ref orientation, ref ratio }) => {
                let children = self.tree.get(id).unwrap().children().clone();

                let left_id = &children[0];
                let right_id = &children[1];

                let left_geo = Geometry {
                    origin: remaining.origin,
                    size: Size {
                        w: (remaining.size.w as f64 *
                            if *orientation == Orientation::Horizontal {
                            *ratio
                        } else {
                            1.0
                        }) as u32,
                        h: (remaining.size.h as f64 *
                            if *orientation == Orientation::Vertical {
                            *ratio
                        } else {
                            1.0
                        }) as u32,
                    },
                };

                let right_geo = Geometry {
                    origin: Point {
                        x: remaining.origin.x +
                           if *orientation == Orientation::Horizontal {
                            left_geo.size.w as i32
                        } else {
                            0
                        },
                        y: remaining.origin.y +
                           if *orientation == Orientation::Vertical {
                            left_geo.size.h as i32
                        } else {
                            0
                        },
                    },
                    size: Size {
                        w: remaining.size.w -
                           if *orientation == Orientation::Horizontal {
                            left_geo.size.w
                        } else {
                            0
                        },
                        h: remaining.size.h -
                           if *orientation == Orientation::Vertical {
                            left_geo.size.h
                        } else {
                            0
                        },
                    },
                };

                self.calculate_node(left_id, left_geo);
                self.calculate_node(right_id, right_geo);
            }
        };
    }

    fn insert_view(&mut self, view: &View) -> bool {
        trace!(slog_scope::logger(), "Inserting new View {:?}", view);

        let new_leaf = Leaf { view: view.weak_reference() };

        let new_node = NodeBuilder::new(Data::Leaf(new_leaf)).build();

        match self.tiling_root() {
            Some(ref current_id) => {
                debug!(slog_scope::logger(),
                       "Insert: Tiling root: {:?}",
                       current_id);
                let new_split = NodeBuilder::new(Data::Split(Split {
                        orientation: self.next_orientation,
                        ratio: 0.5,
                    }))
                    .with_child_capacity(3)
                    .build();

                let split_id = {
                    let parent_id = self.tree.get(current_id).unwrap().parent().cloned();
                    if let Some(parent_id) = parent_id {
                        let direction =
                            self.tree.get(&parent_id).unwrap().direction_of_child(current_id).unwrap();

                        let split_id = self.tree.insert(new_split, UnderNode(&parent_id)).unwrap();
                        self.tree.move_node(current_id, ToParent(&split_id)).unwrap();

                        if direction == Direction::Left {
                            let other_id =
                                self.tree.get(&parent_id).unwrap().other_child(&split_id).unwrap().clone();
                            self.tree.swap_nodes(&split_id, &other_id, TakeChildren).unwrap();
                        }

                        split_id
                    } else {
                        self.tree.insert(new_split, AsRoot).unwrap()
                    }
                };

                view.insert::<NodeId>(self.tree.insert(new_node, UnderNode(&split_id)).unwrap());
            }
            None => {
                debug!(slog_scope::logger(), "Insert: No Tiling root");
                let root_id = self.tree.root_node_id().cloned();
                match root_id {
                    Some(_) => {
                        let new_split = NodeBuilder::new(Data::Split(Split {
                                orientation: self.next_orientation,
                                ratio: 0.5,
                            }))
                            .with_child_capacity(3)
                            .build();

                        let split_id = self.tree.insert(new_split, AsRoot).unwrap();
                        view.insert::<NodeId>(self.tree.insert(new_node, UnderNode(&split_id)).unwrap());
                    }
                    None => {
                        view.insert::<NodeId>(self.tree.insert(new_node, AsRoot).unwrap());
                    }
                };
            }
        };

        true
    }

    fn remove_view(&mut self, view: &View) {
        debug!(slog_scope::logger(), "Removing: {:?}", view);

        if let Some(node_id) = view.remove::<NodeId>() {
            let parent = self.tree.get(&node_id).unwrap().parent().cloned();

            self.tree.remove_node(node_id, DropChildren).unwrap();

            if let Some(parent_id) = parent {
                if self.tree.root_node_id() == Some(&parent_id) {
                    let other_child = self.tree.get(&parent_id).unwrap().children()[0].clone();
                    self.tree.remove_node(parent_id, OrphanChildren).unwrap();
                    self.tree.move_node(&other_child, ToRoot).unwrap();
                } else {
                    let parent_parent_id = self.tree.get(&parent_id).unwrap().parent().unwrap().clone();

                    let direction =
                        self.tree.get(&parent_parent_id).unwrap().direction_of_child(&parent_id).unwrap();

                    self.tree.remove_node(parent_id, LiftChildren).unwrap();

                    if direction == Direction::Left {
                        let children = self.tree.get(&parent_parent_id).unwrap().children().clone();
                        self.tree.swap_nodes(&children[0], &children[1], TakeChildren).unwrap();
                    }
                }
            }
        };
    }

    fn move_focus(&self, view: &View, orientation: Orientation, direction: Direction) {
        if let Some(weakview) = self.relative_to(view, orientation, direction) {
            weakview.run(|new_view| {
                debug!(slog_scope::logger(),
                       "Moving focus from {:?} to {:?}",
                       view,
                       new_view);
                new_view.focus();
            });
        }
    }

    fn move_view(&mut self, view: &View, orientation: Orientation, direction: Direction) {
        let lock = view.get::<NodeId>();
        if let Some(node_id) = lock.as_ref().and_then(|x| x.read().ok()) {
            let parent_id = self.tree.get(&*node_id).unwrap().parent().cloned();
            if let Some(parent_id) = parent_id {
                let position_matches = self.tree.get(&parent_id).unwrap().is_child(&node_id, Some(direction));
                if !position_matches {
                    let other_id = self.tree.get(&parent_id).unwrap().other_child(&node_id).cloned().unwrap();
                    self.tree.swap_nodes(&node_id, &other_id, TakeChildren).unwrap();
                }

                let parent_orientation = self.tree.get(&parent_id).unwrap().orientation().unwrap();

                match parent_orientation {
                    x if x != orientation => {
                        match *self.tree.get_mut(&parent_id).unwrap().data_mut() {
                            Data::Split(Split { ref mut orientation, .. }) => *orientation = !*orientation,
                            _ => unreachable!(),
                        };
                    }
                    x if x == orientation && position_matches => {
                        let parent_parent_id = self.tree.get(&parent_id).unwrap().parent().cloned();
                        if let Some(parent_parent_id) = parent_parent_id {
                            let (child_id, upper_child_id) = (&*node_id,
                                                              self.tree
                                                                  .get(&parent_parent_id)
                                                                  .unwrap()
                                                                  .other_child(&parent_id)
                                                                  .cloned()
                                                                  .unwrap());

                            self.tree.swap_nodes(child_id, &upper_child_id, TakeChildren).unwrap();

                            let data1 = self.tree.get(&parent_id).unwrap().data().clone();
                            let data2 = self.tree.get_mut(&parent_parent_id).unwrap().replace_data(data1);
                            self.tree.get_mut(&parent_id).unwrap().replace_data(data2);

                            let position_matches = self.tree
                                .get(&parent_parent_id)
                                .unwrap()
                                .is_child(child_id, Some(direction));
                            if !position_matches {
                                let other_id = self.tree
                                    .get(&parent_parent_id)
                                    .unwrap()
                                    .other_child(child_id)
                                    .cloned()
                                    .unwrap();
                                self.tree.swap_nodes(child_id, &other_id, TakeChildren).unwrap();
                            }
                        }
                    }
                    _ => {}
                }
            }
        };
    }

    fn resize(&mut self, view: &View, for_orientation: Orientation, for_direction: Direction, by: f64) {
        let lock = view.get::<NodeId>();
        let maybe_node_id = lock.as_ref().and_then(|x| x.read().ok()).map(|x| (*x).clone());
        if let Some(mut node_id) = maybe_node_id {
            let mut maybe_parent_id = self.tree.get(&node_id).unwrap().parent().cloned();

            while let Some(parent_id) = maybe_parent_id {
                let mut parent = self.tree.get_mut(&parent_id).unwrap();
                let data = parent.data().clone();
                match data {
                    Data::Split(Split { orientation, ratio }) => {
                        if for_orientation == orientation {
                            match (for_direction, parent.direction_of_child(&node_id).unwrap()) {
                                (Direction::Left, Direction::Left) |
                                (Direction::Left, Direction::Right) => {
                                    parent.replace_data(Data::Split(Split {
                                        orientation: orientation,
                                        ratio: (ratio - by).max(0.1),
                                    }))
                                }
                                (Direction::Right, Direction::Left) |
                                (Direction::Right, Direction::Right) => {
                                    parent.replace_data(Data::Split(Split {
                                        orientation: orientation,
                                        ratio: (ratio + by).min(0.9),
                                    }))
                                }
                            };
                            maybe_parent_id = None;
                        } else {
                            node_id = parent_id;
                            maybe_parent_id = parent.parent().cloned();
                        }
                    }
                    _ => unreachable!(),
                }
            }
        };
    }

    fn relative_to(&self, view: &View, for_orientation: Orientation, for_direction: Direction)
                   -> Option<WeakView> {
        let lock = view.get::<NodeId>();
        let result = match lock.as_ref().and_then(|x| x.read().ok()) {
            Some(node_id) => {
                let mut node_id = &*node_id;

                let mut maybe_parent_id = self.tree.get(node_id).unwrap().parent();

                while let Some(parent_id) = maybe_parent_id {
                    let parent = self.tree.get(parent_id).unwrap();
                    match *parent.data() {
                        Data::Split(Split { ref orientation, .. }) => {
                            if for_orientation == *orientation &&
                               parent.is_child(node_id, Some(!for_direction)) {
                                return Some(self.tree
                                    .get(parent.other_child(node_id).unwrap())
                                    .unwrap()
                                    .lowest_view(&self.tree, !for_direction));
                            }

                            node_id = parent_id;
                            maybe_parent_id = self.tree.get(node_id).unwrap().parent();
                        }
                        _ => unreachable!(),
                    }
                }

                None
            }
            None => None,
        };
        result
    }

    fn number_of_children(&self) -> usize {
        self.tree
            .root_node_id()
            .map(|root| self.tree.get(root).unwrap().number_of_children(&self.tree))
            .unwrap_or(0)
    }

    // helpful for debugging
    // pub fn print_tree(&self)
    // {
    // let mut format = String::new();
    // self.tree.root_node_id().map(|root|
    // self.tree.get(root).unwrap().print_tree(&self.tree, &mut
    // format)).unwrap_or(());
    // debug!(slog_scope::logger(), "{}", format);
    // }

    fn tiling_root(&self) -> Option<NodeId> {
        match self.tiling_root {
            Some(ref focused) => {
                focused.run(|view| {
                        let lock = view.get::<NodeId>();
                        let result = match lock.as_ref().and_then(|x| x.read().ok()) {
                            Some(node) => Some((*node).clone()),
                            None => None,
                        };
                        result
                    })
                    .and_then(|x| x)
            }
            None => None,
        }
    }
}
