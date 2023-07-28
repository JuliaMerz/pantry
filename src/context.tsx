

import {createContext} from "react";


interface ErrorContextSchema {
  sendError: (err: string) => void;
  lastError: string;
}

export const ErrorContext = createContext<ErrorContextSchema>(
  {} as ErrorContextSchema
);

