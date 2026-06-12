import { toast as sonnerToast } from 'sonner';

interface ToastOptions {
  title: string;
  description?: string;
  variant?: 'default' | 'destructive';
}

export function useToast() {
  const toast = (options: ToastOptions) => {
    if (options.variant === 'destructive') {
      sonnerToast.error(options.title, {
        description: options.description,
      });
    } else {
      sonnerToast.success(options.title, {
        description: options.description,
      });
    }
  };

  return { toast };
}
