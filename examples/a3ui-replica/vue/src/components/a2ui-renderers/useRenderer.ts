import A2Row from './A2Row.vue'
import A2Column from './A2Column.vue'
import A2List from './A2List.vue'
import A2Card from './A2Card.vue'
import A2Text from './A2Text.vue'
import A2Image from './A2Image.vue'
import A2Icon from './A2Icon.vue'
import A2Button from './A2Button.vue'
import A2Tabs from './A2Tabs.vue'
import A2TextField from './A2TextField.vue'
import A2CheckBox from './A2CheckBox.vue'
import A2Slider from './A2Slider.vue'
import A2Divider from './A2Divider.vue'
import A2DashboardCard from './A2DashboardCard.vue'
import A2Title from './A2Title.vue'
import A2Metric from './A2Metric.vue'
import A2Badge from './A2Badge.vue'
import A2PieChart from './A2PieChart.vue'
import A2BarChart from './A2BarChart.vue'
import A2DataTable from './A2DataTable.vue'

const rendererMap: Record<string, any> = {
  Row: A2Row,
  Column: A2Column,
  List: A2List,
  Card: A2Card,
  Text: A2Text,
  Image: A2Image,
  Icon: A2Icon,
  Button: A2Button,
  Tabs: A2Tabs,
  TextField: A2TextField,
  CheckBox: A2CheckBox,
  Slider: A2Slider,
  Divider: A2Divider,
  Video: A2Text,
  AudioPlayer: A2Text,
  DateTimeInput: A2TextField,
  ChoicePicker: A2Tabs,
  Modal: A2Card,
  DashboardCard: A2DashboardCard,
  Title: A2Title,
  Metric: A2Metric,
  Badge: A2Badge,
  PieChart: A2PieChart,
  BarChart: A2BarChart,
  DataTable: A2DataTable,
}

export function useRenderer() {
  function getRenderer(name: string) {
    return rendererMap[name] || A2Text
  }
  return { getRenderer, rendererMap }
}
