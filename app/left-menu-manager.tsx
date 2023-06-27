"use client"

import { useState } from 'react';
import LayerTree from './layer-tree'
import LeftMenu, { LeftMenuItems } from './left-menu'


export default function LeftMenuManager() {
        const [selected, setSelected] = useState<LeftMenuItems>();

    const updateSelected = (menuItem: LeftMenuItems) => {
        console.log("select ", menuItem);
        if (menuItem != selected) {
            setSelected(menuItem);
        } else {
            setSelected(undefined);
        }
    }
    
  return (
      <>
        <LeftMenu selectedItem={selected} updateSelectedItem={updateSelected}/>
        <LayerTree show={selected == LeftMenuItems.Layers}/>
      </>
  )
}
