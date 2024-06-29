export class Cluster {
  items: Set<string>;

  constructor(items: Iterable<string> | undefined) {
    this.items = new Set(items);
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
    let newItems = new Set(this.items);

    otherCluster.forEach((item) => newItems.add(item));

    return new Cluster(newItems);
  }

  intersection(otherCluster: Cluster) {
    let intersection = new Set<string>();
    for (let elem of this.items) {
      if (otherCluster.items.has(elem)) {
        intersection.add(elem);
      }
    }

    return new Cluster(intersection);
  }

  size(): number {
    return this.items.size;
  }
}
