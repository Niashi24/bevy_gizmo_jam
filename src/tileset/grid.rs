use std::fmt::{Display, Formatter};
use bevy::prelude::Reflect;




#[derive(Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub struct Grid<T> {
    grid: Vec<Vec<T>>,
    pub w: usize,
    pub h: usize,
}

impl<T> Grid<T> {
    pub fn new(grid: Vec<Vec<T>>) -> Self {
        let h = grid.len();
        let w = grid.first().map(Vec::len).unwrap_or(0);
        Self {
            grid,
            w,
            h
        }
    }
    
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.grid.get(y).and_then(|y| y.get(x))
    }
    
    #[inline]
    #[allow(dead_code)]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.grid.get_mut(y).and_then(|y| y.get_mut(x))
    }

    #[allow(dead_code)]
    pub fn get_i(&self, x: i64, y: i64) -> Option<&T> {
        if x < 0 || y < 0 { None }
        else { self.get(x as usize, y as usize) }
    }

    #[allow(dead_code)]
    pub fn get_cycle(&self, mut x: i64, mut y: i64) -> Option<&T> {
        x = x.rem_euclid(self.w as i64);
        y = y.rem_euclid(self.h as i64);
        self.get(x as usize, y as usize)
    }

    #[allow(dead_code)]
    pub fn positions<FN: Fn(&T) -> bool>(&self, predicate: FN) -> Vec<(usize, usize)> {
        let mut pos = vec![];
        for (y, row) in self.grid.iter().enumerate() {
            for (x, item) in row.iter().enumerate() {
                if predicate(item) {
                    pos.push((x, y));
                }
            }
        }
        pos
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> GridIter<T> {
        GridIter {
            grid: self,
            x: 0,
            y: 0,
        }
    }

    #[allow(dead_code)]
    pub fn map<X, FN: Fn(T) -> X>(self, func: FN) -> Grid<X> {
        self.grid.into_iter().map(|x| x.into_iter().map(&func)).collect()
    }

    #[allow(dead_code)]
    pub fn try_from_iter<E, IT, TIT>(iter: TIT) -> Result<Self, (usize, usize, E)>
    where
        IT: IntoIterator<Item = Result<T, E>>,
        TIT: IntoIterator<Item = IT>,
    {
        iter.into_iter().enumerate()
            .map(move |(y, inner)| {
                inner.into_iter().enumerate().map(|(x, item)| {
                    item.map_err(|e| (x, y, e))
                }).collect::<Result<_, _>>()
            }).collect::<Result<_, _>>()
            .map(Self::new)
    }
}

pub struct GridIter<'a, T> {
    grid: &'a Grid<T>,
    x: usize,
    y: usize,
}

impl<'a, T> Iterator for GridIter<'a, T> {
    type Item = ((usize, usize), &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.grid.get(self.x, self.y)
            .map(|x| ((self.x, self.y), x));
        
        self.x += 1;
        if self.x == self.grid.w {
            self.y += 1;
            self.x = 0;
            if self.y == self.grid.h {
                return None;
            }
        }
        
        item
    }
}

impl<T, IT> FromIterator<IT> for Grid<T>
    where
        IT: IntoIterator<Item = T>
{
    fn from_iter<TIT: IntoIterator<Item=IT>>(iter: TIT) -> Self {
        Self::new(iter.into_iter()
            .map(|y| y.into_iter()
                .collect())
            .collect())
    }
}
//
// impl<T, E, IT, TIT> TryFrom<TIT> for Grid<T>
//     where
//         IT: IntoIterator<Item = Result<T, E>>,
//         TIT: IntoIterator<Item = IT>,
// {
//     type Error = (usize, usize, E);
//
//     fn try_from(value: TIT) -> Result<Self, Self::Error> {
//         value.into_iter().enumerate()
//             .map(move |(y, inner)| {
//                 inner.into_iter().enumerate().map(|(x, item)| {
//                     item.map_err(|e| (x, y, e))
//                 }).collect::<Result<_, _>>()
//             }).collect::<Result<_, _>>()
//             .map(Self::new)
//     }
// }

// impl <T, IT, E> FromIterator<T> for Result<Grid<T>, E>
//     where
//         IT: IntoIterator<Item = Result<T, E>>
// {
//     fn from_iter<TIT: IntoIterator<Item=IT>>(iter: TIT) -> Self {
//         iter.into_iter()
//             .map(|y| y.into_iter()
//                 .collect::<Result<_, _>>())
//             .collect::<Result<_, _>>()
//             .map(Self::new)
//     }
// }

impl<T: Display> Display for Grid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in self.grid.iter() {
            for col in row.iter() {
                write!(f, "{}", col)?;
            }
            writeln!(f)?;
        }
        
        Ok(())
    }
}

