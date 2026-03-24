import { Button } from '@kit.ArkUI';
import { Text } from '@kit.ArkUI';
import { Column } from '@kit.ArkUI';
import { ForEach } from '@kit.ArkUI';

interface TutorialItemItem {
    id: number
    title: string
}

@Preview
@Component
export struct TutorialItem {
    @Consume('pathStack') pathStack: NavPathStack

    @State items: TutorialItemItem[] = [{ id: 1, title: "Hello", brief: "B", imageSrc: "C" },
        { id: 2, title: "World", brief: "E", imageSrc: "F" }
    ]

    build() {
        Column() {
            ForEach(this.items, (item: TutorialItemItem) => {
                Text(item.title)
            }, (item: TutorialItemItem): string => item.id)
            .onClick(() => {
                this.pathStack.pushPathByName('articleDetail', item)
            })
        }
    }
}
