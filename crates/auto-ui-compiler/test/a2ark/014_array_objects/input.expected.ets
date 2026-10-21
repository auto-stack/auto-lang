import { Button } from '@kit.ArkUI';

interface TestArrayObjectsItem {
  brief: string
  id: string
  imageSrc: string
  title: string
}

@Preview
@Component
export struct TestArrayObjects {
    @State items: TestArrayObjectsItem[] = [{id: '0', brief: "B", imageSrc: "C", title: "A"}, {id: '1', brief: "E", imageSrc: "F", title: "D"}]

  build() {
        Column() {
          ForEach(this.items, (item: TestArrayObjectsItem) => {
            Text('test')
          }, (item: TestArrayObjectsItem): string => item.id)
        }
  }
}
