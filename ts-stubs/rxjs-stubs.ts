// RxJS stubs for TypeScript checking
export class Observable<T = any> {
  constructor(subscribe?: (observer: Observer<T>) => TeardownLogic) {}
  
  pipe<A>(op1: OperatorFunction<T, A>): Observable<A>;
  pipe<A, B>(op1: OperatorFunction<T, A>, op2: OperatorFunction<A, B>): Observable<B>;
  pipe<A, B, C>(op1: OperatorFunction<T, A>, op2: OperatorFunction<A, B>, op3: OperatorFunction<B, C>): Observable<C>;
  pipe(...operations: OperatorFunction<any, any>[]): Observable<any> {
    return new Observable();
  }
}

export interface Observer<T> {
  next: (value: T) => void;
  error: (err: any) => void;
  complete: () => void;
}

export interface TeardownLogic {
  unsubscribe(): void;
}

export interface OperatorFunction<T, R> {
  (source: Observable<T>): Observable<R>;
}

// Add missing RxJS functions
export function lastValueFrom<T>(source: Observable<T>): Promise<T> {
  return Promise.resolve({} as T);
}

export function firstValueFrom<T>(source: Observable<T>): Promise<T> {
  return Promise.resolve({} as T);
}
