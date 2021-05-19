using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
#if UNITY_EDITOR
    using UnityEditor;
#endif
using UnityEngine;
using UnityEngine.UI;

[ExecuteInEditMode]
 public class Tablet : MonoBehaviour
 {
     public Color ObjectColor;
 
     private Color currentColor;
     private Material materialColored;
 
     void Update()
     {
         if (ObjectColor != currentColor)
         {
            #if UNITY_EDITOR
                //helps stop memory leaks
                if (materialColored != null)
                    UnityEditor.AssetDatabase.DeleteAsset(UnityEditor.AssetDatabase.GetAssetPath(materialColored));
            #endif
 
             //create a new material
             materialColored = new Material(Shader.Find("Diffuse"));
             materialColored.color = currentColor = ObjectColor;
             this.GetComponent<Renderer>().material = materialColored;
         }
     }
 }