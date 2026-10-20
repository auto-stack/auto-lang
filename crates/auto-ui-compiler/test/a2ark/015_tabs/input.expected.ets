import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestTabs {
    @State activeTab: Object = 'tab1'

  @State currentIndex: number = 0
  private tabsController: TabsController = new TabsController()

    @Builder
    tabBarBuilder(title: string, targetIndex: number, selectedIcon?: Resource, unselectIcon?: Resource) {
      Column() {
        if (selectedIcon && unselectIcon) {
          Image(this.currentIndex === targetIndex ? selectedIcon : unselectIcon)
            .width(24)
            .height(24)
        }
        Text(title)
          .fontFamily('HarmonyHeiTi-Medium')
          .fontSize(10)
          .fontColor(this.currentIndex === targetIndex ? '#0A59F7' : 'rgba(0,0,0,0.60)')
          .textAlign(TextAlign.Center)
          .lineHeight(14)
          .fontWeight(500)
      }
      .width('100%')
      .height('100%')
      .justifyContent(FlexAlign.Center)
      .alignItems(HorizontalAlign.Center)
      .onClick(() => {
        this.currentIndex = targetIndex
        this.tabsController.changeIndex(targetIndex)
      })
    }

  build() {
        Column() {
        Tabs({ barPosition: BarPosition.End, controller: this.tabsController }) {
          TabContent() {
                  Text('Content 1')
          }
          .tabBar(this.tabBarBuilder('Tab 1', 0))
          TabContent() {
                  Text('Content 2')
          }
          .tabBar(this.tabBarBuilder('Tab 2', 1))
        }
        .vertical(false)
        .scrollable(false)
        .backgroundColor('#F1F3F5')
        }
  }
}
