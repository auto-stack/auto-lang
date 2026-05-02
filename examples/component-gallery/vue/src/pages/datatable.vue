<!-- DatatablePage component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { DropdownMenu } from '@/components/ui/dropdown-menu'
import { Input } from '@/components/ui/input'
import { Select } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'


const activeTab = ref<string>('preview')
const sortColumn = ref<string>('')
const sortDirection = ref<string>('asc')
const filterText = ref<string>('')
const selectedRows = ref<number>(0)
const currentPage = ref<number>(1)

const copiedCode = ref<string>('')
const showDatatableBasicCode = ref<boolean>(true)
const activeDatatableBasicTab = ref<string>('auto')
const showDatatableSortingCode = ref<boolean>(true)
const activeDatatableSortingTab = ref<string>('auto')
const showDatatableFilteringCode = ref<boolean>(true)
const activeDatatableFilteringTab = ref<string>('auto')
const showDatatablePaginationCode = ref<boolean>(true)
const activeDatatablePaginationTab = ref<string>('auto')
const showDatatableSelectionCode = ref<boolean>(true)
const activeDatatableSelectionTab = ref<string>('auto')
const showDatatableActionsCode = ref<boolean>(true)
const activeDatatableActionsTab = ref<string>('auto')
const showDatatableVisibilityCode = ref<boolean>(true)
const activeDatatableVisibilityTab = ref<string>('auto')
const showDatatableCompleteCode = ref<boolean>(true)
const activeDatatableCompleteTab = ref<string>('auto')

// Copy to clipboard function
async function copyCode(code: string, id: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(code)
    copiedCode.value = id
    setTimeout(() => {
      copiedCode.value = ''
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

const datatableBasicAutoCode = `table {
    table-header {
        table-row {
            table-head (text: "Status") {}
            table-head (text: "Email") {}
            table-head (class: "text-right", text: "Amount") {}
        }
    }
    table-body {
        table-row {
            table-cell {
                badge (variant: "default", text: "Success") {}
            }
            table-cell (class: "lowercase", text: "ken99@yahoo.com") {}
            table-cell (text: "$316.00", class: "text-right font-medium") {}
        }
        table-row {
            table-cell {
                badge (variant: "secondary", text: "Pending") {}
            }
            table-cell (text: "abe45@gmail.com", class: "lowercase") {}
            table-cell (class: "text-right font-medium", text: "$242.00") {}
        }
        table-row {
            table-cell {
                badge (variant: "outline", text: "Processing") {}
            }
            table-cell (text: "monserrat44@gmail.com", class: "lowercase") {}
            table-cell (class: "text-right font-medium", text: "$837.00") {}
        }
    }
}
`
const datatableBasicVueCode = `<Table>
  <TableHeader>
    <TableRow>
      <TableHead>Status</TableHead>
      <TableHead>Email</TableHead>
      <TableHead class="text-right">Amount</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    <TableRow>
      <TableCell>
        <Badge>Success</Badge>
      </TableCell>
      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
      <TableCell class="text-right font-medium">$316.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell>
        <Badge>Pending</Badge>
      </TableCell>
      <TableCell class="lowercase">abe45@gmail.com</TableCell>
      <TableCell class="text-right font-medium">$242.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell>
        <Badge>Processing</Badge>
      </TableCell>
      <TableCell class="lowercase">monserrat44@gmail.com</TableCell>
      <TableCell class="text-right font-medium">$837.00</TableCell>
    </TableRow>
  </TableBody>
</Table>
`
const datatableSortingAutoCode = `table {
    table-header {
        table-row {
            table-head {
                button (variant: "ghost", class: "-ml-4", onclick: ..ToggleSortStatus) {
                    "Status"
                    if .sortColumn == "status" {
                        if .sortDirection == "asc" {
                            icon (class: "ml-2 h-4 w-4", name: "arrow-up") {}
                        }
                        if .sortDirection == "desc" {
                            icon (class: "ml-2 h-4 w-4", name: "arrow-down") {}
                        }
                    }
                    if .sortColumn != "status" {
                        icon (class: "ml-2 h-4 w-4", name: "arrow-up-down") {}
                    }
                }
            }
            table-head {
                button (class: "-ml-4", variant: "ghost", onclick: ..ToggleSortEmail) {
                    "Email"
                    if .sortColumn == "email" {
                        if .sortDirection == "asc" {
                            icon (name: "arrow-up", class: "ml-2 h-4 w-4") {}
                        }
                        if .sortDirection == "desc" {
                            icon (name: "arrow-down", class: "ml-2 h-4 w-4") {}
                        }
                    }
                    if .sortColumn != "email" {
                        icon (name: "arrow-up-down", class: "ml-2 h-4 w-4") {}
                    }
                }
            }
            table-head (class: "text-right") {
                button (variant: "ghost", class: "-ml-4", onclick: ..ToggleSortAmount) {
                    "Amount"
                    if .sortColumn == "amount" {
                        if .sortDirection == "asc" {
                            icon (class: "ml-2 h-4 w-4", name: "arrow-up") {}
                        }
                        if .sortDirection == "desc" {
                            icon (name: "arrow-down", class: "ml-2 h-4 w-4") {}
                        }
                    }
                    if .sortColumn != "amount" {
                        icon (class: "ml-2 h-4 w-4", name: "arrow-up-down") {}
                    }
                }
            }
        }
    }
    table-body {
        if .sortColumn == "email" {
            if .sortDirection == "asc" {
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (text: "abe45@gmail.com", class: "lowercase") {}
                    table-cell (text: "$242.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Failed", variant: "destructive") {}
                    }
                    table-cell (text: "carmella@hotmail.com", class: "lowercase") {}
                    table-cell (text: "$721.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (class: "lowercase", text: "ken99@yahoo.com") {}
                    table-cell (class: "text-right font-medium", text: "$316.00") {}
                }
            }
            if .sortDirection == "desc" {
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (text: "ken99@yahoo.com", class: "lowercase") {}
                    table-cell (text: "$316.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Failed", variant: "destructive") {}
                    }
                    table-cell (text: "carmella@hotmail.com", class: "lowercase") {}
                    table-cell (class: "text-right font-medium", text: "$721.00") {}
                }
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                    table-cell (text: "$242.00", class: "text-right font-medium") {}
                }
            }
        }
        if .sortColumn == "amount" {
            if .sortDirection == "asc" {
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                    table-cell (text: "$242.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (class: "lowercase", text: "ken99@yahoo.com") {}
                    table-cell (class: "text-right font-medium", text: "$316.00") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Failed", variant: "destructive") {}
                    }
                    table-cell (class: "lowercase", text: "carmella@hotmail.com") {}
                    table-cell (class: "text-right font-medium", text: "$721.00") {}
                }
            }
            if .sortDirection == "desc" {
                table-row {
                    table-cell {
                        badge (variant: "destructive", text: "Failed") {}
                    }
                    table-cell (class: "lowercase", text: "carmella@hotmail.com") {}
                    table-cell (class: "text-right font-medium", text: "$721.00") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (text: "ken99@yahoo.com", class: "lowercase") {}
                    table-cell (text: "$316.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                    table-cell (class: "text-right font-medium", text: "$242.00") {}
                }
            }
        }
        if .sortColumn == "status" {
            if .sortDirection == "asc" {
                table-row {
                    table-cell {
                        badge (text: "Failed", variant: "destructive") {}
                    }
                    table-cell (text: "carmella@hotmail.com", class: "lowercase") {}
                    table-cell (class: "text-right font-medium", text: "$721.00") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (text: "abe45@gmail.com", class: "lowercase") {}
                    table-cell (class: "text-right font-medium", text: "$242.00") {}
                }
                table-row {
                    table-cell {
                        badge (variant: "default", text: "Success") {}
                    }
                    table-cell (text: "ken99@yahoo.com", class: "lowercase") {}
                    table-cell (text: "$316.00", class: "text-right font-medium") {}
                }
            }
            if .sortDirection == "desc" {
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (text: "abe45@gmail.com", class: "lowercase") {}
                    table-cell (text: "$242.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Success", variant: "default") {}
                    }
                    table-cell (class: "lowercase", text: "ken99@yahoo.com") {}
                    table-cell (text: "$316.00", class: "text-right font-medium") {}
                }
                table-row {
                    table-cell {
                        badge (text: "Failed", variant: "destructive") {}
                    }
                    table-cell (class: "lowercase", text: "carmella@hotmail.com") {}
                    table-cell (text: "$721.00", class: "text-right font-medium") {}
                }
            }
        }
        if .sortColumn == "" {
            table-row {
                table-cell {
                    badge (variant: "default", text: "Success") {}
                }
                table-cell (class: "lowercase", text: "ken99@yahoo.com") {}
                table-cell (text: "$316.00", class: "text-right font-medium") {}
            }
            table-row {
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                table-cell (text: "$242.00", class: "text-right font-medium") {}
            }
            table-row {
                table-cell {
                    badge (variant: "destructive", text: "Failed") {}
                }
                table-cell (text: "carmella@hotmail.com", class: "lowercase") {}
                table-cell (class: "text-right font-medium", text: "$721.00") {}
            }
        }
    }
}
`
const datatableSortingVueCode = `<Table>
  <TableHeader>
    <TableRow>
      <TableHead>
        <Button variant="ghost" @click="onToggleSortStatus">
          Status
          <template v-if="sortColumn == 'status'">
            <template v-if="sortDirection == 'asc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
            </template>
            <template v-if="sortDirection == 'desc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
            </template>
          </template>
          <template v-if="sortColumn != 'status'">
            <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
          </template>
        </Button>
      </TableHead>
      <TableHead>
        <Button variant="ghost" @click="onToggleSortEmail">
          Email
          <template v-if="sortColumn == 'email'">
            <template v-if="sortDirection == 'asc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
            </template>
            <template v-if="sortDirection == 'desc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
            </template>
          </template>
          <template v-if="sortColumn != 'email'">
            <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
          </template>
        </Button>
      </TableHead>
      <TableHead class="text-right">
        <Button variant="ghost" @click="onToggleSortAmount">
          Amount
          <template v-if="sortColumn == 'amount'">
            <template v-if="sortDirection == 'asc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
            </template>
            <template v-if="sortDirection == 'desc'">
              <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
            </template>
          </template>
          <template v-if="sortColumn != 'amount'">
            <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
          </template>
        </Button>
      </TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    <template v-if="sortColumn == 'email'">
      <template v-if="sortDirection == 'asc'">
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
      </template>
      <template v-if="sortDirection == 'desc'">
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
      </template>
    </template>
    <template v-if="sortColumn == 'amount'">
      <template v-if="sortDirection == 'asc'">
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
      </template>
      <template v-if="sortDirection == 'desc'">
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
      </template>
    </template>
    <template v-if="sortColumn == 'status'">
      <template v-if="sortDirection == 'asc'">
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
      </template>
      <template v-if="sortDirection == 'desc'">
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">abe45@gmail.com</TableCell>
          <TableCell class="text-right font-medium">$242.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Success</Badge>
          </TableCell>
          <TableCell class="lowercase">ken99@yahoo.com</TableCell>
          <TableCell class="text-right font-medium">$316.00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            <Badge>Failed</Badge>
          </TableCell>
          <TableCell class="lowercase">carmella@hotmail.com</TableCell>
          <TableCell class="text-right font-medium">$721.00</TableCell>
        </TableRow>
      </template>
    </template>
    <template v-if="sortColumn == ''">
      <TableRow>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">ken99@yahoo.com</TableCell>
        <TableCell class="text-right font-medium">$316.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">abe45@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$242.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell>
          <Badge>Failed</Badge>
        </TableCell>
        <TableCell class="lowercase">carmella@hotmail.com</TableCell>
        <TableCell class="text-right font-medium">$721.00</TableCell>
      </TableRow>
    </template>
  </TableBody>
</Table>
`
const datatableFilteringAutoCode = `col (gap: "4") {
    row (class: "flex items-center gap-2") {
        text (class: "text-sm font-medium", text: "Filter:") {}
        input (class: "w-[250px]", placeholder: "Search by email...") {}
    }
    table {
        table-header {
            table-row {
                table-head (text: "Status") {}
                table-head (text: "Email") {}
                table-head (text: "Amount", class: "text-right") {}
            }
        }
        table-body {
            table-row {
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (text: "ken99@yahoo.com", class: "lowercase") {}
                table-cell (text: "$316.00", class: "text-right font-medium") {}
            }
            table-row {
                table-cell {
                    badge (variant: "default", text: "Success") {}
                }
                table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                table-cell (text: "$242.00", class: "text-right font-medium") {}
            }
        }
    }
}
`
const datatableFilteringVueCode = `<div class="flex flex-col gap-4">
  <div class="flex flex-row gap-4 flex items-center gap-2">
    <span class="text-muted-foreground leading-7 text-sm font-medium">Filter:</span>
    <Input placeholder="Search by email..." />
  </div>
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>Status</TableHead>
        <TableHead>Email</TableHead>
        <TableHead class="text-right">Amount</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">ken99@yahoo.com</TableCell>
        <TableCell class="text-right font-medium">$316.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">abe45@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$242.00</TableCell>
      </TableRow>
    </TableBody>
  </Table>
</div>
`
const datatablePaginationAutoCode = `col (gap: "4") {
    table {
        table-header {
            table-row {
                table-head (text: "Invoice") {}
                table-head (text: "Status") {}
                table-head (text: "Amount", class: "text-right") {}
            }
        }
        table-body {
            table-row {
                table-cell (class: "font-medium", text: "INV001") {}
                table-cell (text: "Paid") {}
                table-cell (class: "text-right", text: "$250.00") {}
            }
            table-row {
                table-cell (text: "INV002", class: "font-medium") {}
                table-cell (text: "Pending") {}
                table-cell (class: "text-right", text: "$150.00") {}
            }
        }
    }
    row (class: "flex items-center justify-between") {
        row (class: "flex items-center gap-2") {
            text (class: "text-sm text-muted-foreground", text: "Rows per page") {}
            select (class: "w-[70px]") {
                select-trigger {
                    select-value (text: "10") {}
                }
                select-content {
                    select-item (text: "10", value: "10") {}
                    select-item (text: "20", value: "20") {}
                }
            }
        }
        text (text: "Page 1 of 10", class: "text-sm text-muted-foreground") {}
        row (gap: "2") {
            button (size: "icon", disabled: true, variant: "outline") {
                icon (name: "chevron-left", class: "h-4 w-4") {}
            }
            button (variant: "outline", size: "icon") {
                icon (name: "chevron-right", class: "h-4 w-4") {}
            }
        }
    }
}
`
const datatablePaginationVueCode = `<div class="flex flex-col gap-4">
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>Invoice</TableHead>
        <TableHead>Status</TableHead>
        <TableHead class="text-right">Amount</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow>
        <TableCell class="font-medium">INV001</TableCell>
        <TableCell>Paid</TableCell>
        <TableCell class="text-right">$250.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="font-medium">INV002</TableCell>
        <TableCell>Pending</TableCell>
        <TableCell class="text-right">$150.00</TableCell>
      </TableRow>
    </TableBody>
  </Table>
  <div class="flex flex-row gap-4 flex items-center justify-between">
    <div class="flex flex-row gap-4 flex items-center gap-2">
      <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Rows per page</span>
      <Select>
        <div>
          <div>10</div>
        </div>
        <div>
          <div value="10">10</div>
          <div value="20">20</div>
        </div>
      </Select>
    </div>
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Page 1 of 10</span>
    <div class="flex flex-row gap-2">
      <Button variant="outline" size="icon" disabled>
        <span class="w-5 h-5 h-4 w-4" name="chevron-left" />
      </Button>
      <Button variant="outline" size="icon">
        <span class="w-5 h-5 h-4 w-4" name="chevron-right" />
      </Button>
    </div>
  </div>
</div>
`
const datatableSelectionAutoCode = `col (gap: "4") {
    table {
        table-header {
            table-row {
                table-head (class: "w-[50px]") {
                    checkbox {}
                }
                table-head (text: "Status") {}
                table-head (text: "Email") {}
                table-head (text: "Amount", class: "text-right") {}
            }
        }
        table-body {
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox {}
                }
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (text: "ken99@yahoo.com") {}
                table-cell (text: "$316.00", class: "text-right font-medium") {}
            }
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox (checked: true) {}
                }
                table-cell {
                    badge (variant: "default", text: "Success") {}
                }
                table-cell (text: "abe45@gmail.com") {}
                table-cell (text: "$242.00", class: "text-right font-medium") {}
            }
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox {}
                }
                table-cell {
                    badge (variant: "outline", text: "Processing") {}
                }
                table-cell (text: "monserrat44@gmail.com") {}
                table-cell (text: "$837.00", class: "text-right font-medium") {}
            }
        }
    }
    row (class: "flex items-center justify-between") {
        text (class: "text-sm text-muted-foreground", text: "1 of 3 row(s) selected.") {}
        row (gap: "2") {
            button (text: "Previous", size: "sm", variant: "outline") {}
            button (variant: "outline", text: "Next", size: "sm") {}
        }
    }
}
`
const datatableSelectionVueCode = `<div class="flex flex-col gap-4">
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead class="w-[50px]">
          <Checkbox />
        </TableHead>
        <TableHead>Status</TableHead>
        <TableHead>Email</TableHead>
        <TableHead class="text-right">Amount</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox />
        </TableCell>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell>ken99@yahoo.com</TableCell>
        <TableCell class="text-right font-medium">$316.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox :default-checked="true" />
        </TableCell>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell>abe45@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$242.00</TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox />
        </TableCell>
        <TableCell>
          <Badge>Processing</Badge>
        </TableCell>
        <TableCell>monserrat44@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$837.00</TableCell>
      </TableRow>
    </TableBody>
  </Table>
  <div class="flex flex-row gap-4 flex items-center justify-between">
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 of 3 row(s) selected.</span>
    <div class="flex flex-row gap-2">
      <Button variant="outline" size="sm">Previous</Button>
      <Button variant="outline" size="sm">Next</Button>
    </div>
  </div>
</div>
`
const datatableActionsAutoCode = `table {
    table-header {
        table-row {
            table-head (text: "Invoice") {}
            table-head (text: "Status") {}
            table-head (text: "Amount", class: "text-right") {}
            table-head (text: "", class: "w-[50px]") {}
        }
    }
    table-body {
        table-row {
            table-cell (text: "INV001", class: "font-medium") {}
            table-cell {
                badge (text: "Paid", variant: "default") {}
            }
            table-cell (text: "$250.00", class: "text-right") {}
            table-cell (class: "w-[50px]") {
                dropdown-menu {
                    dropdown-menu-trigger {
                        button (variant: "ghost", size: "icon") {
                            icon (name: "more-horizontal", class: "h-4 w-4") {}
                        }
                    }
                    dropdown-menu-content {
                        dropdown-menu-item (text: "Copy ID") {}
                        dropdown-menu-item (text: "View details") {}
                        dropdown-menu-separator {}
                        dropdown-menu-item (class: "text-destructive", text: "Delete") {}
                    }
                }
            }
        }
        table-row {
            table-cell (class: "font-medium", text: "INV002") {}
            table-cell {
                badge (text: "Pending", variant: "secondary") {}
            }
            table-cell (text: "$150.00", class: "text-right") {}
            table-cell (class: "w-[50px]") {
                dropdown-menu {
                    dropdown-menu-trigger {
                        button (variant: "ghost", size: "icon") {
                            icon (class: "h-4 w-4", name: "more-horizontal") {}
                        }
                    }
                    dropdown-menu-content {
                        dropdown-menu-item (text: "Copy ID") {}
                        dropdown-menu-item (text: "View details") {}
                    }
                }
            }
        }
    }
}
`
const datatableActionsVueCode = `<Table>
  <TableHeader>
    <TableRow>
      <TableHead>Invoice</TableHead>
      <TableHead>Status</TableHead>
      <TableHead class="text-right">Amount</TableHead>
      <TableHead class="w-[50px]"></TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    <TableRow>
      <TableCell class="font-medium">INV001</TableCell>
      <TableCell>
        <Badge>Paid</Badge>
      </TableCell>
      <TableCell class="text-right">$250.00</TableCell>
      <TableCell class="w-[50px]">
        <DropdownMenu>
          <div>
            <Button variant="ghost" size="icon">
              <span class="w-5 h-5 h-4 w-4" name="more-horizontal" />
            </Button>
          </div>
          <div>
            <div>Copy ID</div>
            <div>View details</div>
            <div />
            <div class="text-destructive">Delete</div>
          </div>
        </DropdownMenu>
      </TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV002</TableCell>
      <TableCell>
        <Badge>Pending</Badge>
      </TableCell>
      <TableCell class="text-right">$150.00</TableCell>
      <TableCell class="w-[50px]">
        <DropdownMenu>
          <div>
            <Button variant="ghost" size="icon">
              <span class="w-5 h-5 h-4 w-4" name="more-horizontal" />
            </Button>
          </div>
          <div>
            <div>Copy ID</div>
            <div>View details</div>
          </div>
        </DropdownMenu>
      </TableCell>
    </TableRow>
  </TableBody>
</Table>
`
const datatableVisibilityAutoCode = `col (gap: "4") {
    row (class: "flex items-center justify-between") {
        input (class: "w-[200px]", placeholder: "Search...") {}
        dropdown-menu {
            dropdown-menu-trigger {
                button (variant: "outline", size: "sm") {
                    icon (class: "mr-2 h-4 w-4", name: "columns") {}
                    "Columns"
                }
            }
            dropdown-menu-content {
                dropdown-menu-item (text: "Status") {}
                dropdown-menu-item (text: "Email") {}
                dropdown-menu-item (text: "Amount") {}
            }
        }
    }
    table {
        table-header {
            table-row {
                table-head (text: "Status") {}
                table-head (text: "Email") {}
                table-head (class: "text-right", text: "Amount") {}
            }
        }
        table-body {
            table-row {
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (text: "ken99@yahoo.com") {}
                table-cell (class: "text-right font-medium", text: "$316.00") {}
            }
        }
    }
}
`
const datatableVisibilityVueCode = `<div class="flex flex-col gap-4">
  <div class="flex flex-row gap-4 flex items-center justify-between">
    <Input placeholder="Search..." />
    <DropdownMenu>
      <div>
        <Button variant="outline" size="sm">
          <span class="w-5 h-5 mr-2 h-4 w-4" name="columns" />
          Columns
        </Button>
      </div>
      <div>
        <div>Status</div>
        <div>Email</div>
        <div>Amount</div>
      </div>
    </DropdownMenu>
  </div>
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>Status</TableHead>
        <TableHead>Email</TableHead>
        <TableHead class="text-right">Amount</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell>ken99@yahoo.com</TableCell>
        <TableCell class="text-right font-medium">$316.00</TableCell>
      </TableRow>
    </TableBody>
  </Table>
</div>
`
const datatableCompleteAutoCode = `col (gap: "4") {
    row (class: "flex items-center justify-between") {
        input (class: "w-[200px]", placeholder: "Filter emails...") {}
        row (gap: "2") {
            dropdown-menu {
                dropdown-menu-trigger {
                    button (size: "sm", variant: "outline") {
                        icon (class: "mr-2 h-4 w-4", name: "columns") {}
                        "View"
                    }
                }
                dropdown-menu-content (align: "end") {
                    dropdown-menu-item (text: "Status") {}
                    dropdown-menu-item (text: "Email") {}
                    dropdown-menu-item (text: "Amount") {}
                }
            }
            button (variant: "outline", size: "sm") {
                icon (class: "mr-2 h-4 w-4", name: "plus") {}
                "Add Payment"
            }
        }
    }
    table {
        table-header {
            table-row {
                table-head (class: "w-[50px]") {
                    checkbox {}
                }
                table-head {
                    button (class: "-ml-4", variant: "ghost") {
                        "Status"
                        icon (name: "arrow-up-down", class: "ml-2 h-4 w-4") {}
                    }
                }
                table-head {
                    button (variant: "ghost", class: "-ml-4") {
                        "Email"
                        icon (class: "ml-2 h-4 w-4", name: "arrow-up-down") {}
                    }
                }
                table-head (text: "Amount", class: "text-right") {}
                table-head (text: "", class: "w-[50px]") {}
            }
        }
        table-body {
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox {}
                }
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (text: "ken99@yahoo.com", class: "lowercase") {}
                table-cell (class: "text-right font-medium", text: "$316.00") {}
                table-cell (class: "w-[50px]") {
                    button (size: "icon", variant: "ghost") {
                        "..."
                    }
                }
            }
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox (checked: true) {}
                }
                table-cell {
                    badge (text: "Success", variant: "default") {}
                }
                table-cell (class: "lowercase", text: "abe45@gmail.com") {}
                table-cell (text: "$242.00", class: "text-right font-medium") {}
                table-cell (class: "w-[50px]") {
                    button (size: "icon", variant: "ghost") {
                        "..."
                    }
                }
            }
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox {}
                }
                table-cell {
                    badge (text: "Processing", variant: "outline") {}
                }
                table-cell (class: "lowercase", text: "monserrat44@gmail.com") {}
                table-cell (class: "text-right font-medium", text: "$837.00") {}
                table-cell (class: "w-[50px]") {
                    button (size: "icon", variant: "ghost") {
                        "..."
                    }
                }
            }
            table-row {
                table-cell (class: "w-[50px]") {
                    checkbox {}
                }
                table-cell {
                    badge (text: "Failed", variant: "destructive") {}
                }
                table-cell (class: "lowercase", text: "carmella@hotmail.com") {}
                table-cell (class: "text-right font-medium", text: "$721.00") {}
                table-cell (class: "w-[50px]") {
                    button (variant: "ghost", size: "icon") {
                        "..."
                    }
                }
            }
        }
    }
    row (class: "flex items-center justify-between") {
        text (text: "1 of 4 row(s) selected.", class: "text-sm text-muted-foreground") {}
        row (class: "flex items-center gap-6") {
            row (class: "flex items-center gap-2") {
                text (class: "text-sm text-muted-foreground", text: "Rows per page") {}
                select (class: "w-[70px]") {
                    select-trigger {
                        select-value {}
                    }
                    select-content {
                        select-item (value: "5", text: "5") {}
                        select-item (value: "10", text: "10") {}
                    }
                }
            }
            text (class: "text-sm text-muted-foreground", text: "Page 1 of 1") {}
            row (gap: "2") {
                button (variant: "outline", size: "icon", disabled: true) {
                    icon (name: "chevron-left", class: "h-4 w-4") {}
                }
                button (variant: "outline", size: "icon", disabled: true) {
                    icon (name: "chevron-right", class: "h-4 w-4") {}
                }
            }
        }
    }
}
`
const datatableCompleteVueCode = `<div class="flex flex-col gap-4">
  <div class="flex flex-row gap-4 flex items-center justify-between">
    <Input placeholder="Filter emails..." />
    <div class="flex flex-row gap-2">
      <DropdownMenu>
        <div>
          <Button variant="outline" size="sm">
            <span class="w-5 h-5 mr-2 h-4 w-4" name="columns" />
            View
          </Button>
        </div>
        <div align="end">
          <div>Status</div>
          <div>Email</div>
          <div>Amount</div>
        </div>
      </DropdownMenu>
      <Button variant="outline" size="sm">
        <span class="w-5 h-5 mr-2 h-4 w-4" name="plus" />
        Add Payment
      </Button>
    </div>
  </div>
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead class="w-[50px]">
          <Checkbox />
        </TableHead>
        <TableHead>
          <Button variant="ghost">
            Status
            <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
          </Button>
        </TableHead>
        <TableHead>
          <Button variant="ghost">
            Email
            <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
          </Button>
        </TableHead>
        <TableHead class="text-right">Amount</TableHead>
        <TableHead class="w-[50px]"></TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox />
        </TableCell>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">ken99@yahoo.com</TableCell>
        <TableCell class="text-right font-medium">$316.00</TableCell>
        <TableCell class="w-[50px]">
          <Button variant="ghost" size="icon">
            ...
          </Button>
        </TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox :default-checked="true" />
        </TableCell>
        <TableCell>
          <Badge>Success</Badge>
        </TableCell>
        <TableCell class="lowercase">abe45@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$242.00</TableCell>
        <TableCell class="w-[50px]">
          <Button variant="ghost" size="icon">
            ...
          </Button>
        </TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox />
        </TableCell>
        <TableCell>
          <Badge>Processing</Badge>
        </TableCell>
        <TableCell class="lowercase">monserrat44@gmail.com</TableCell>
        <TableCell class="text-right font-medium">$837.00</TableCell>
        <TableCell class="w-[50px]">
          <Button variant="ghost" size="icon">
            ...
          </Button>
        </TableCell>
      </TableRow>
      <TableRow>
        <TableCell class="w-[50px]">
          <Checkbox />
        </TableCell>
        <TableCell>
          <Badge>Failed</Badge>
        </TableCell>
        <TableCell class="lowercase">carmella@hotmail.com</TableCell>
        <TableCell class="text-right font-medium">$721.00</TableCell>
        <TableCell class="w-[50px]">
          <Button variant="ghost" size="icon">
            ...
          </Button>
        </TableCell>
      </TableRow>
    </TableBody>
  </Table>
  <div class="flex flex-row gap-4 flex items-center justify-between">
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 of 4 row(s) selected.</span>
    <div class="flex flex-row gap-4 flex items-center gap-6">
      <div class="flex flex-row gap-4 flex items-center gap-2">
        <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Rows per page</span>
        <Select>
          <div>
            <div />
          </div>
          <div>
            <div value="5">5</div>
            <div value="10">10</div>
          </div>
        </Select>
      </div>
      <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Page 1 of 1</span>
      <div class="flex flex-row gap-2">
        <Button variant="outline" size="icon" disabled>
          <span class="w-5 h-5 h-4 w-4" name="chevron-left" />
        </Button>
        <Button variant="outline" size="icon" disabled>
          <span class="w-5 h-5 h-4 w-4" name="chevron-right" />
        </Button>
      </div>
    </div>
  </div>
</div>
`
const codeblock1Code = `npx shadcn-vue@latest add table`
const codeblock2Code = `npm install @tanstack/vue-table`
watch(activeDatatableBasicTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeDatatableSortingTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeDatatableFilteringTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeDatatablePaginationTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeDatatableSelectionTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeDatatableActionsTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeDatatableVisibilityTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeDatatableCompleteTab, () => {
  nextTick(() => Prism.highlightAll())
})
onMounted(() => {
  nextTick(() => Prism.highlightAll())
})


</script>

<template>
  <div class="flex flex-col">
    <div class="flex flex-col gap-4">
      <h1 class="text-4xl font-bold tracking-tight">Data Table</h1>
      Powerful table and datagrids built using TanStack Table.
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Introduction</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">The Data Table is not a single component but a guide on how to build your own data tables using TanStack Table (@tanstack/vue-table) combined with the Table component. This allows for maximum flexibility with sorting, filtering, pagination, and more.</span>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Installation</h2>
      <span class="text-muted-foreground leading-7 font-medium">1. Add the Table component:</span>
      <div class="relative rounded-lg border overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 bg-muted border-b">
          <span class="text-xs text-muted-foreground font-medium">bash</span>
          <button
            @click="copyCode(codeblock1Code, 'codeblock1')"
            class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
          >
            <svg v-if="copiedCode !== 'codeblock1'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
            {{ copiedCode === 'codeblock1' ? 'Copied!' : 'Copy' }}
          </button>
        </div>
        <pre class="p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))] overflow-x-auto"><code class="block font-mono !p-0 language-bash">{{ codeblock1Code }}</code></pre>
      </div>
      <span class="text-muted-foreground leading-7 font-medium">2. Add TanStack Table dependency:</span>
      <div class="relative rounded-lg border overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 bg-muted border-b">
          <span class="text-xs text-muted-foreground font-medium">bash</span>
          <button
            @click="copyCode(codeblock2Code, 'codeblock2')"
            class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
          >
            <svg v-if="copiedCode !== 'codeblock2'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
            {{ copiedCode === 'codeblock2' ? 'Copied!' : 'Copy' }}
          </button>
        </div>
        <pre class="p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))] overflow-x-auto"><code class="block font-mono !p-0 language-bash">{{ codeblock2Code }}</code></pre>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Basic Table</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Start with a basic table to display your data.</span>
      <!-- Merged DatatableBasic Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Status</TableHead>
                  <TableHead>Email</TableHead>
                  <TableHead class="text-right">Amount</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell>
                    <Badge>Success</Badge>
                  </TableCell>
                  <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                  <TableCell class="text-right font-medium">$316.00</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>
                    <Badge>Pending</Badge>
                  </TableCell>
                  <TableCell class="lowercase">abe45@gmail.com</TableCell>
                  <TableCell class="text-right font-medium">$242.00</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>
                    <Badge>Processing</Badge>
                  </TableCell>
                  <TableCell class="lowercase">monserrat44@gmail.com</TableCell>
                  <TableCell class="text-right font-medium">$837.00</TableCell>
                </TableRow>
              </TableBody>
            </Table>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showDatatableBasicCode = !showDatatableBasicCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showDatatableBasicCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showDatatableBasicCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeDatatableBasicTab = 'auto'"
                  :class="activeDatatableBasicTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeDatatableBasicTab = 'vue'"
                  :class="activeDatatableBasicTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeDatatableBasicTab === 'auto' ? datatableBasicAutoCode : datatableBasicVueCode, 'datatable-basic')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'datatable-basic'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'datatable-basic' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeDatatableBasicTab === 'auto' ? 'auto' : 'html')">{{ activeDatatableBasicTab === 'auto' ? datatableBasicAutoCode : datatableBasicVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Sorting</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Click column headers to sort ascending or descending. Try clicking on each column!</span>
      <!-- Merged DatatableSorting Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>
                    <Button variant="ghost" @click="onToggleSortStatus">
                      Status
                      <template v-if="sortColumn == 'status'">
                        <template v-if="sortDirection == 'asc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
                        </template>
                        <template v-if="sortDirection == 'desc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
                        </template>
                      </template>
                      <template v-if="sortColumn != 'status'">
                        <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
                      </template>
                    </Button>
                  </TableHead>
                  <TableHead>
                    <Button variant="ghost" @click="onToggleSortEmail">
                      Email
                      <template v-if="sortColumn == 'email'">
                        <template v-if="sortDirection == 'asc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
                        </template>
                        <template v-if="sortDirection == 'desc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
                        </template>
                      </template>
                      <template v-if="sortColumn != 'email'">
                        <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
                      </template>
                    </Button>
                  </TableHead>
                  <TableHead class="text-right">
                    <Button variant="ghost" @click="onToggleSortAmount">
                      Amount
                      <template v-if="sortColumn == 'amount'">
                        <template v-if="sortDirection == 'asc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up" />
                        </template>
                        <template v-if="sortDirection == 'desc'">
                          <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-down" />
                        </template>
                      </template>
                      <template v-if="sortColumn != 'amount'">
                        <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
                      </template>
                    </Button>
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <template v-if="sortColumn == 'email'">
                  <template v-if="sortDirection == 'asc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                  </template>
                  <template v-if="sortDirection == 'desc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                  </template>
                </template>
                <template v-if="sortColumn == 'amount'">
                  <template v-if="sortDirection == 'asc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                  </template>
                  <template v-if="sortDirection == 'desc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                  </template>
                </template>
                <template v-if="sortColumn == 'status'">
                  <template v-if="sortDirection == 'asc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                  </template>
                  <template v-if="sortDirection == 'desc'">
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">abe45@gmail.com</TableCell>
                      <TableCell class="text-right font-medium">$242.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Success</Badge>
                      </TableCell>
                      <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                      <TableCell class="text-right font-medium">$316.00</TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell>
                        <Badge>Failed</Badge>
                      </TableCell>
                      <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                      <TableCell class="text-right font-medium">$721.00</TableCell>
                    </TableRow>
                  </template>
                </template>
                <template v-if="sortColumn == ''">
                  <TableRow>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                    <TableCell class="text-right font-medium">$316.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">abe45@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$242.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>
                      <Badge>Failed</Badge>
                    </TableCell>
                    <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                    <TableCell class="text-right font-medium">$721.00</TableCell>
                  </TableRow>
                </template>
              </TableBody>
            </Table>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showDatatableSortingCode = !showDatatableSortingCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showDatatableSortingCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showDatatableSortingCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeDatatableSortingTab = 'auto'"
                  :class="activeDatatableSortingTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeDatatableSortingTab = 'vue'"
                  :class="activeDatatableSortingTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeDatatableSortingTab === 'auto' ? datatableSortingAutoCode : datatableSortingVueCode, 'datatable-sorting')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'datatable-sorting'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'datatable-sorting' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeDatatableSortingTab === 'auto' ? 'auto' : 'html')">{{ activeDatatableSortingTab === 'auto' ? datatableSortingAutoCode : datatableSortingVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <div>
        <ToggleSortStatus />
        <div>
          <template v-if="sortColumn == 'status'">
            <div />
            <div />
            <template v-if="sortDirection == 'asc'">
              <div />
            </template>
            <template v-else>
              <div />
            </template>
          </template>
          <template v-else>
            <div />
            <div>status</div>
            <div />
            <div>asc</div>
          </template>
        </div>
        <ToggleSortEmail />
        <div>
          <template v-if="sortColumn == 'email'">
            <div />
            <div />
            <template v-if="sortDirection == 'asc'">
              <div />
            </template>
            <template v-else>
              <div />
            </template>
          </template>
          <template v-else>
            <div />
            <div>email</div>
            <div />
            <div>asc</div>
          </template>
        </div>
        <ToggleSortAmount />
        <div>
          <template v-if="sortColumn == 'amount'">
            <div />
            <div />
            <template v-if="sortDirection == 'asc'">
              <div />
            </template>
            <template v-else>
              <div />
            </template>
          </template>
          <template v-else>
            <div />
            <div>amount</div>
            <div />
            <div>asc</div>
          </template>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Filtering</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Search and filter data across columns.</span>
      <!-- Merged DatatableFiltering Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="flex flex-col gap-4">
              <div class="flex flex-row gap-4 flex items-center gap-2">
                <span class="text-muted-foreground leading-7 text-sm font-medium">Filter:</span>
                <Input placeholder="Search by email..." />
              </div>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Status</TableHead>
                    <TableHead>Email</TableHead>
                    <TableHead class="text-right">Amount</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                    <TableCell class="text-right font-medium">$316.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">abe45@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$242.00</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showDatatableFilteringCode = !showDatatableFilteringCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showDatatableFilteringCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showDatatableFilteringCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeDatatableFilteringTab = 'auto'"
                  :class="activeDatatableFilteringTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeDatatableFilteringTab = 'vue'"
                  :class="activeDatatableFilteringTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeDatatableFilteringTab === 'auto' ? datatableFilteringAutoCode : datatableFilteringVueCode, 'datatable-filtering')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'datatable-filtering'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'datatable-filtering' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeDatatableFilteringTab === 'auto' ? 'auto' : 'html')">{{ activeDatatableFilteringTab === 'auto' ? datatableFilteringAutoCode : datatableFilteringVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Pagination</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Navigate through large datasets with page controls.</span>
      <!-- Merged DatatablePagination Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="flex flex-col gap-4">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Invoice</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead class="text-right">Amount</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell class="font-medium">INV001</TableCell>
                    <TableCell>Paid</TableCell>
                    <TableCell class="text-right">$250.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="font-medium">INV002</TableCell>
                    <TableCell>Pending</TableCell>
                    <TableCell class="text-right">$150.00</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              <div class="flex flex-row gap-4 flex items-center justify-between">
                <div class="flex flex-row gap-4 flex items-center gap-2">
                  <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Rows per page</span>
                  <Select>
                    <div>
                      <div>10</div>
                    </div>
                    <div>
                      <div value="10">10</div>
                      <div value="20">20</div>
                    </div>
                  </Select>
                </div>
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Page 1 of 10</span>
                <div class="flex flex-row gap-2">
                  <Button variant="outline" size="icon" disabled>
                    <span class="w-5 h-5 h-4 w-4" name="chevron-left" />
                  </Button>
                  <Button variant="outline" size="icon">
                    <span class="w-5 h-5 h-4 w-4" name="chevron-right" />
                  </Button>
                </div>
              </div>
            </div>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showDatatablePaginationCode = !showDatatablePaginationCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showDatatablePaginationCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showDatatablePaginationCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeDatatablePaginationTab = 'auto'"
                  :class="activeDatatablePaginationTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeDatatablePaginationTab = 'vue'"
                  :class="activeDatatablePaginationTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeDatatablePaginationTab === 'auto' ? datatablePaginationAutoCode : datatablePaginationVueCode, 'datatable-pagination')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'datatable-pagination'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'datatable-pagination' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeDatatablePaginationTab === 'auto' ? 'auto' : 'html')">{{ activeDatatablePaginationTab === 'auto' ? datatablePaginationAutoCode : datatablePaginationVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Row Selection</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Select rows with checkboxes for bulk operations.</span>
      <!-- Merged DatatableSelection Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="flex flex-col gap-4">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead class="w-[50px]">
                      <Checkbox />
                    </TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Email</TableHead>
                    <TableHead class="text-right">Amount</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox />
                    </TableCell>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell>ken99@yahoo.com</TableCell>
                    <TableCell class="text-right font-medium">$316.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox :default-checked="true" />
                    </TableCell>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell>abe45@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$242.00</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox />
                    </TableCell>
                    <TableCell>
                      <Badge>Processing</Badge>
                    </TableCell>
                    <TableCell>monserrat44@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$837.00</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              <div class="flex flex-row gap-4 flex items-center justify-between">
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 of 3 row(s) selected.</span>
                <div class="flex flex-row gap-2">
                  <Button variant="outline" size="sm">Previous</Button>
                  <Button variant="outline" size="sm">Next</Button>
                </div>
              </div>
            </div>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showDatatableSelectionCode = !showDatatableSelectionCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showDatatableSelectionCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showDatatableSelectionCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeDatatableSelectionTab = 'auto'"
                  :class="activeDatatableSelectionTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeDatatableSelectionTab = 'vue'"
                  :class="activeDatatableSelectionTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeDatatableSelectionTab === 'auto' ? datatableSelectionAutoCode : datatableSelectionVueCode, 'datatable-selection')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'datatable-selection'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'datatable-selection' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeDatatableSelectionTab === 'auto' ? 'auto' : 'html')">{{ activeDatatableSelectionTab === 'auto' ? datatableSelectionAutoCode : datatableSelectionVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Row Actions</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Add action menus to each row.</span>
      <!-- Merged DatatableActions Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Invoice</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead class="text-right">Amount</TableHead>
                  <TableHead class="w-[50px]"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell class="font-medium">INV001</TableCell>
                  <TableCell>
                    <Badge>Paid</Badge>
                  </TableCell>
                  <TableCell class="text-right">$250.00</TableCell>
                  <TableCell class="w-[50px]">
                    <DropdownMenu>
                      <div>
                        <Button variant="ghost" size="icon">
                          <span class="w-5 h-5 h-4 w-4" name="more-horizontal" />
                        </Button>
                      </div>
                      <div>
                        <div>Copy ID</div>
                        <div>View details</div>
                        <div />
                        <div class="text-destructive">Delete</div>
                      </div>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell class="font-medium">INV002</TableCell>
                  <TableCell>
                    <Badge>Pending</Badge>
                  </TableCell>
                  <TableCell class="text-right">$150.00</TableCell>
                  <TableCell class="w-[50px]">
                    <DropdownMenu>
                      <div>
                        <Button variant="ghost" size="icon">
                          <span class="w-5 h-5 h-4 w-4" name="more-horizontal" />
                        </Button>
                      </div>
                      <div>
                        <div>Copy ID</div>
                        <div>View details</div>
                      </div>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showDatatableActionsCode = !showDatatableActionsCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showDatatableActionsCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showDatatableActionsCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeDatatableActionsTab = 'auto'"
                  :class="activeDatatableActionsTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeDatatableActionsTab = 'vue'"
                  :class="activeDatatableActionsTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeDatatableActionsTab === 'auto' ? datatableActionsAutoCode : datatableActionsVueCode, 'datatable-actions')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'datatable-actions'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'datatable-actions' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeDatatableActionsTab === 'auto' ? 'auto' : 'html')">{{ activeDatatableActionsTab === 'auto' ? datatableActionsAutoCode : datatableActionsVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Column Visibility</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Toggle column visibility with a dropdown menu.</span>
      <!-- Merged DatatableVisibility Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="flex flex-col gap-4">
              <div class="flex flex-row gap-4 flex items-center justify-between">
                <Input placeholder="Search..." />
                <DropdownMenu>
                  <div>
                    <Button variant="outline" size="sm">
                      <span class="w-5 h-5 mr-2 h-4 w-4" name="columns" />
                      Columns
                    </Button>
                  </div>
                  <div>
                    <div>Status</div>
                    <div>Email</div>
                    <div>Amount</div>
                  </div>
                </DropdownMenu>
              </div>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Status</TableHead>
                    <TableHead>Email</TableHead>
                    <TableHead class="text-right">Amount</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell>ken99@yahoo.com</TableCell>
                    <TableCell class="text-right font-medium">$316.00</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showDatatableVisibilityCode = !showDatatableVisibilityCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showDatatableVisibilityCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showDatatableVisibilityCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeDatatableVisibilityTab = 'auto'"
                  :class="activeDatatableVisibilityTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeDatatableVisibilityTab = 'vue'"
                  :class="activeDatatableVisibilityTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeDatatableVisibilityTab === 'auto' ? datatableVisibilityAutoCode : datatableVisibilityVueCode, 'datatable-visibility')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'datatable-visibility'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'datatable-visibility' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeDatatableVisibilityTab === 'auto' ? 'auto' : 'html')">{{ activeDatatableVisibilityTab === 'auto' ? datatableVisibilityAutoCode : datatableVisibilityVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Complete Example</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">A full-featured data table with all features combined.</span>
      <!-- Merged DatatableComplete Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="flex flex-col gap-4">
              <div class="flex flex-row gap-4 flex items-center justify-between">
                <Input placeholder="Filter emails..." />
                <div class="flex flex-row gap-2">
                  <DropdownMenu>
                    <div>
                      <Button variant="outline" size="sm">
                        <span class="w-5 h-5 mr-2 h-4 w-4" name="columns" />
                        View
                      </Button>
                    </div>
                    <div align="end">
                      <div>Status</div>
                      <div>Email</div>
                      <div>Amount</div>
                    </div>
                  </DropdownMenu>
                  <Button variant="outline" size="sm">
                    <span class="w-5 h-5 mr-2 h-4 w-4" name="plus" />
                    Add Payment
                  </Button>
                </div>
              </div>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead class="w-[50px]">
                      <Checkbox />
                    </TableHead>
                    <TableHead>
                      <Button variant="ghost">
                        Status
                        <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
                      </Button>
                    </TableHead>
                    <TableHead>
                      <Button variant="ghost">
                        Email
                        <span class="w-5 h-5 ml-2 h-4 w-4" name="arrow-up-down" />
                      </Button>
                    </TableHead>
                    <TableHead class="text-right">Amount</TableHead>
                    <TableHead class="w-[50px]"></TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox />
                    </TableCell>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">ken99@yahoo.com</TableCell>
                    <TableCell class="text-right font-medium">$316.00</TableCell>
                    <TableCell class="w-[50px]">
                      <Button variant="ghost" size="icon">
                        ...
                      </Button>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox :default-checked="true" />
                    </TableCell>
                    <TableCell>
                      <Badge>Success</Badge>
                    </TableCell>
                    <TableCell class="lowercase">abe45@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$242.00</TableCell>
                    <TableCell class="w-[50px]">
                      <Button variant="ghost" size="icon">
                        ...
                      </Button>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox />
                    </TableCell>
                    <TableCell>
                      <Badge>Processing</Badge>
                    </TableCell>
                    <TableCell class="lowercase">monserrat44@gmail.com</TableCell>
                    <TableCell class="text-right font-medium">$837.00</TableCell>
                    <TableCell class="w-[50px]">
                      <Button variant="ghost" size="icon">
                        ...
                      </Button>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="w-[50px]">
                      <Checkbox />
                    </TableCell>
                    <TableCell>
                      <Badge>Failed</Badge>
                    </TableCell>
                    <TableCell class="lowercase">carmella@hotmail.com</TableCell>
                    <TableCell class="text-right font-medium">$721.00</TableCell>
                    <TableCell class="w-[50px]">
                      <Button variant="ghost" size="icon">
                        ...
                      </Button>
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              <div class="flex flex-row gap-4 flex items-center justify-between">
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">1 of 4 row(s) selected.</span>
                <div class="flex flex-row gap-4 flex items-center gap-6">
                  <div class="flex flex-row gap-4 flex items-center gap-2">
                    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Rows per page</span>
                    <Select>
                      <div>
                        <div />
                      </div>
                      <div>
                        <div value="5">5</div>
                        <div value="10">10</div>
                      </div>
                    </Select>
                  </div>
                  <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Page 1 of 1</span>
                  <div class="flex flex-row gap-2">
                    <Button variant="outline" size="icon" disabled>
                      <span class="w-5 h-5 h-4 w-4" name="chevron-left" />
                    </Button>
                    <Button variant="outline" size="icon" disabled>
                      <span class="w-5 h-5 h-4 w-4" name="chevron-right" />
                    </Button>
                  </div>
                </div>
              </div>
            </div>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showDatatableCompleteCode = !showDatatableCompleteCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showDatatableCompleteCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showDatatableCompleteCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeDatatableCompleteTab = 'auto'"
                  :class="activeDatatableCompleteTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeDatatableCompleteTab = 'vue'"
                  :class="activeDatatableCompleteTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeDatatableCompleteTab === 'auto' ? datatableCompleteAutoCode : datatableCompleteVueCode, 'datatable-complete')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'datatable-complete'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'datatable-complete' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeDatatableCompleteTab === 'auto' ? 'auto' : 'html')">{{ activeDatatableCompleteTab === 'auto' ? datatableCompleteAutoCode : datatableCompleteVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Features</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">The Data Table guide covers:</span>
      <div class="flex flex-col gap-2 py-2">
        <div class="flex flex-row gap-2">
          <Badge>Sorting</Badge>
          <Badge>Filtering</Badge>
          <Badge>Pagination</Badge>
        </div>
        <div class="flex flex-row gap-2">
          <Badge>Row Selection</Badge>
          <Badge>Column Visibility</Badge>
          <Badge>Row Actions</Badge>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Table Components</h2>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Component</TableHead>
            <TableHead>Description</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>Table</TableCell>
            <TableCell>Main table container</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>TableHeader</TableCell>
            <TableCell>Header section</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>TableBody</TableCell>
            <TableCell>Body section</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>TableRow</TableCell>
            <TableCell>Table row</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>TableHead</TableCell>
            <TableCell>Header cell</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>TableCell</TableCell>
            <TableCell>Data cell</TableCell>
          </TableRow>
        </TableBody>
      </Table>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">TanStack Table API</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Key API methods from @tanstack/vue-table:</span>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Method</TableHead>
            <TableHead>Description</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell class="font-mono text-sm">useVueTable</TableCell>
            <TableCell>Main composable to create table instance</TableCell>
          </TableRow>
          <TableRow>
            <TableCell class="font-mono text-sm">getCoreRowModel</TableCell>
            <TableCell>Returns core row model</TableCell>
          </TableRow>
          <TableRow>
            <TableCell class="font-mono text-sm">getPaginationRowModel</TableCell>
            <TableCell>Returns pagination row model</TableCell>
          </TableRow>
          <TableRow>
            <TableCell class="font-mono text-sm">getSortedRowModel</TableCell>
            <TableCell>Returns sorted row model</TableCell>
          </TableRow>
          <TableRow>
            <TableCell class="font-mono text-sm">getFilteredRowModel</TableCell>
            <TableCell>Returns filtered row model</TableCell>
          </TableRow>
        </TableBody>
      </Table>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Documentation</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">For the complete guide with code examples, see the shadcn-vue Data Table documentation.</span>
    </div>
  </div>

</template>

<style scoped>
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>


