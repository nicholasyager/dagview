function intersection<T>(set: Set<T>, otherSet: Set<T>): Set<T> {
  let intersection = new Set<T>();
  for (let elem of set) {
    if (otherSet.has(elem)) {
      intersection.add(elem);
    }
  }
  return intersection;
}

function union<T>(set: Set<T>, otherSet: Set<T>): Set<T> {
  let newItems = new Set(set);

  otherSet.forEach((item) => newItems.add(item));
  return newItems;
}

export class Cluster {
  items: Set<string>;
  parents: Set<string>;

  constructor(items: Iterable<string>, parents: Iterable<string>) {
    this.items = new Set(items);
    this.parents = new Set(parents);
  }

  forEach(func: (element: any) => void) {
    return this.items.forEach((element) => {
      func(element);
    });
  }

  getId() {
    return Array.from(this.items).join('-');
  }

  union(otherCluster: Cluster): Cluster {
    return new Cluster(
      union(this.items, otherCluster.items),
      union(this.parents, otherCluster.parents)
    );
  }

  intersection(otherCluster: Cluster) {
    return new Cluster(
      intersection(this.items, otherCluster.items),
      intersection(this.parents, otherCluster.parents)
    );
  }

  size(): number {
    return this.items.size;
  }
}
