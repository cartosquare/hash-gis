"use client"

import { createContext, PropsWithChildren, useContext, useState } from "react";
import { Model } from "../types";

type modelType = {
    model: Model | null,
    setModel: (m: Model) => void;
}

const modelContextDefaultValue: modelType = {
    model: null,
    setModel: (m: Model) => {},
};

// create context
const ModelContext = createContext<modelType>(modelContextDefaultValue);

// use context
export const useModel = (): modelType => {
    return useContext(ModelContext);
}

// create a provider function
export const ModelProvider = (props: PropsWithChildren) => {
    const [model, setModel] = useState<Model | null>(null);

    return (
        <ModelContext.Provider value={{ model, setModel }}>
            {props.children}
        </ModelContext.Provider>
    );
};