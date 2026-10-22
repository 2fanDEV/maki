import Axios, { AxiosRequestConfig , AxiosError }  from 'axios';

import.meta.env
export const AXIOS_INSTANCE = Axios.create({
  baseURL: `http://${import.meta.env.VITE_API_URL}`
});

export const customInstance = <T>(
  config: AxiosRequestConfig,
  options?: AxiosRequestConfig,
): Promise<T> => {
  const promise = AXIOS_INSTANCE({
    ...config,
    ...options,
  }).then(({ data }) => data);

  return promise;
};

// Override the return error type for react-query and swr
export type ErrorType<Error> = AxiosError<Error>;

// Standard body type
export type BodyType<BodyData> = BodyData;