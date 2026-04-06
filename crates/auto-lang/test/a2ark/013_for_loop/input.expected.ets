import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestForLoop {
    @State items: Object = ["A", "B", "C"]

  build() {
        Column() {
          ForEach(this.items, (item: Object) => {
            Text()
    /* Unknown component: item */
          }, (item: Object): string => item.id)
        }
  }
}
