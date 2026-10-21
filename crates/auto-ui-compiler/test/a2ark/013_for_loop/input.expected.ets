import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestForLoop {
    @State items: Object[] = ["A", "B", "C"]

  build() {
        Column() {
          ForEach(this.items, (item: Object) => {
            Text(item)
          }, (item: Object): string => item.id)
        }
  }
}
