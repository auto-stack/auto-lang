import { FieldContextKey } from "vee-validate"
import { computed, inject } from "vue"
import { useId } from "reka-ui"
import { FORM_ITEM_INJECTION_KEY } from "./injectionKeys"

// AutoUI form-item 契约：form-label/form-control/form-description/
// form-message 可在无 vee-validate Field 的 model-bound 表单中直接使用
// （生成的表单页不含 vee-validate 包装）。shadcn-vue 原版在此 throw，
// 与本契约冲突——Form 页整页渲染失败即此。缺 FieldContext 时降级为
// 惰性默认：仅丢失校验态回显，id 关联与布局不受影响。
export function useFormField() {
  const fieldContext = inject(FieldContextKey, undefined)
  const fieldItemContext = inject(FORM_ITEM_INJECTION_KEY, undefined)

  const id = fieldItemContext ?? useId()

  if (!fieldContext) {
    return {
      id,
      name: "",
      formItemId: `${id}-form-item`,
      formDescriptionId: `${id}-form-item-description`,
      formMessageId: `${id}-form-item-message`,
      valid: computed(() => true),
      isDirty: computed(() => false),
      isTouched: computed(() => false),
      error: undefined,
    }
  }

  const { name, errorMessage: error, meta } = fieldContext

  const fieldState = {
    valid: computed(() => meta.valid),
    isDirty: computed(() => meta.dirty),
    isTouched: computed(() => meta.touched),
    error,
  }

  return {
    id,
    name,
    formItemId: `${id}-form-item`,
    formDescriptionId: `${id}-form-item-description`,
    formMessageId: `${id}-form-item-message`,
    ...fieldState,
  }
}
