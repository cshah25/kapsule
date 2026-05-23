export type ToastType = 'info' | 'success' | 'error';

export interface Toast {
  id: number;
  message: string;
  type: ToastType;
}

class ToastStore {
  toasts = $state<Toast[]>([]);
  private nextId = 0;

  add(message: string, type: ToastType = 'info', durationMs = 3000) {
    const id = this.nextId++;
    this.toasts.push({ id, message, type });
    setTimeout(() => {
      this.remove(id);
    }, durationMs);
  }

  error(message: string) {
    this.add(message, 'error', 5000);
  }

  success(message: string) {
    this.add(message, 'success');
  }

  remove(id: number) {
    this.toasts = this.toasts.filter(t => t.id !== id);
  }
}

export const toast = new ToastStore();
