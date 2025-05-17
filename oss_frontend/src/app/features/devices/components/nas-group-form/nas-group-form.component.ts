import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { NasService } from '../../../../shared/services/nas.service';
import { NasGroup } from '../../../../shared/models/nas.model';

@Component({
  selector: 'app-nas-group-form',
  templateUrl: './nas-group-form.component.html',
  styleUrls: ['./nas-group-form.component.scss']
})
export class NasGroupFormComponent implements OnInit {
  groupForm: FormGroup;
  isEditMode = false;
  groupId: number | null = null;
  loading = false;
  submitting = false;
  error = '';
  parentGroups: NasGroup[] = [];

  constructor(
    private fb: FormBuilder,
    private nasService: NasService,
    private route: ActivatedRoute,
    private router: Router
  ) {
    this.groupForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      description: [''],
      parent: [null]
    });
  }

  ngOnInit(): void {
    this.loading = true;

    // Load all NAS groups for parent selection
    this.nasService.getAllNasGroups()
      .subscribe({
        next: (groups) => {
          this.parentGroups = groups;
          
          // Check if we're in edit mode
          const id = this.route.snapshot.paramMap.get('id');
          if (id && id !== 'new') {
            this.isEditMode = true;
            this.groupId = +id;
            this.loadGroupDetails(+id);
          } else {
            this.loading = false;
          }
        },
        error: (err) => {
          this.error = 'Failed to load NAS groups. Please try again later.';
          console.error('Error loading NAS groups:', err);
          this.loading = false;
        }
      });
  }

  loadGroupDetails(id: number): void {
    this.nasService.getNasGroupById(id)
      .subscribe({
        next: (group) => {
          // Populate the form with group details
          this.groupForm.patchValue({
            name: group.name,
            description: group.description,
            parent: group.parent
          });
          
          // Filter out the current group and its children from parent options
          // to prevent circular references
          this.filterParentOptions(id);
          
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load group details. Please try again later.';
          console.error('Error loading group details:', err);
          this.loading = false;
        }
      });
  }

  filterParentOptions(currentGroupId: number): void {
    // This is a simplified approach. In a real app, you would need to
    // recursively filter out all descendants of the current group.
    this.parentGroups = this.parentGroups.filter(group => group.id !== currentGroupId);
  }

  onSubmit(): void {
    if (this.groupForm.invalid) {
      // Mark all fields as touched to trigger validation messages
      Object.keys(this.groupForm.controls).forEach(key => {
        const control = this.groupForm.get(key);
        control?.markAsTouched();
      });
      return;
    }

    this.submitting = true;
    this.error = '';

    const groupData = this.groupForm.value;

    if (this.isEditMode && this.groupId) {
      // Update existing group
      this.nasService.updateNasGroup(this.groupId, groupData)
        .subscribe({
          next: () => {
            this.router.navigate(['/devices/groups']);
          },
          error: (err) => {
            this.error = 'Failed to update NAS group. Please try again later.';
            console.error('Error updating NAS group:', err);
            this.submitting = false;
          }
        });
    } else {
      // Create new group
      this.nasService.createNasGroup(groupData)
        .subscribe({
          next: () => {
            this.router.navigate(['/devices/groups']);
          },
          error: (err) => {
            this.error = 'Failed to create NAS group. Please try again later.';
            console.error('Error creating NAS group:', err);
            this.submitting = false;
          }
        });
    }
  }

  cancel(): void {
    this.router.navigate(['/devices/groups']);
  }
}