// RxJS operators stubs for TypeScript checking
import { Observable, OperatorFunction } from './rxjs-stubs';

export function map<T, R>(project: (value: T, index?: number) => R): OperatorFunction<T, R> {
  return (source: Observable<T>) => new Observable<R>();
}

export function catchError<T, R>(selector: (err: any, caught: Observable<T>) => Observable<R>): OperatorFunction<T, T | R> {
  return (source: Observable<T>) => new Observable<T | R>();
}

export function tap<T>(observer?: Partial<Observer<T>>): OperatorFunction<T, T>;
export function tap<T>(next: (value: T) => void): OperatorFunction<T, T>;
export function tap<T>(next?: any): OperatorFunction<T, T> {
  return (source: Observable<T>) => new Observable<T>();
}

export function switchMap<T, R>(project: (value: T, index: number) => Observable<R>): OperatorFunction<T, R> {
  return (source: Observable<T>) => new Observable<R>();
}

export function mergeMap<T, R>(project: (value: T, index: number) => Observable<R>): OperatorFunction<T, R> {
  return (source: Observable<T>) => new Observable<R>();
}

export function filter<T>(predicate: (value: T, index: number) => boolean): OperatorFunction<T, T> {
  return (source: Observable<T>) => new Observable<T>();
}

export function take<T>(count: number): OperatorFunction<T, T> {
  return (source: Observable<T>) => new Observable<T>();
}

export function first<T>(): OperatorFunction<T, T> {
  return (source: Observable<T>) => new Observable<T>();
}

interface Observer<T> {
  next: (value: T) => void;
  error: (err: any) => void;
  complete: () => void;
}
