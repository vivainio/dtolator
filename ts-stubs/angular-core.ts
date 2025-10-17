// Angular Core stubs for TypeScript checking
export interface ModuleWithProviders<T = any> {
  ngModule: any;
  providers?: any[];
}

export interface Injectable {
  providedIn?: 'root' | 'platform' | 'any' | null;
}

export function Injectable(options?: Injectable): (target: any) => any {
  return (target: any) => target;
}

export class Type<T = any> {
  constructor(public name: string) {}
}

export interface OnInit {
  ngOnInit(): void;
}

export interface OnDestroy {
  ngOnDestroy(): void;
}

export class EventEmitter<T = any> {
  emit(value?: T): void {}
  subscribe(next?: (value: T) => void, error?: (error: any) => void, complete?: () => void): any {}
}

export function Component(options: any): (target: any) => any {
  return (target: any) => target;
}

export function Directive(options: any): (target: any) => any {
  return (target: any) => target;
}

export function Input(bindingPropertyName?: string): any {
  return function (target: any, propertyKey: string) {};
}

export function Output(bindingPropertyName?: string): any {
  return function (target: any, propertyKey: string) {};
}
